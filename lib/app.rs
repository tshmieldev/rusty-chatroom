use std::{error::Error, io, time::Duration};

use mio::{
    Events, Interest, Poll, Token,
    net::{TcpListener, TcpStream},
};
use slotmap::{Key, KeyData, SlotMap, new_key_type};

use crate::connection::{Connection, ReadStatus};
use crate::context::Context;
use crate::queues::Queues;
use crate::{operation::Operation, server::Server};

pub type OnConnectHandler<S> = fn(ctx: &mut Context<'_, S>);
pub type OnDisconnectHandler<S> = fn(ctx: &mut Context<'_, S>);
pub type OnMessageHandler<S> = fn(ctx: &mut Context<'_, S>, message: &str);

pub struct Handlers<S: Server> {
    pub on_connect: OnConnectHandler<S>,
    pub on_message: OnMessageHandler<S>,
    pub on_disconnect: OnDisconnectHandler<S>,
}

impl<S: Server> Default for Handlers<S> {
    fn default() -> Self {
        Self {
            on_connect: |_| {},
            on_message: |_, _| {},
            on_disconnect: |_| {},
        }
    }
}

new_key_type! { struct ClientKey; }

const _: () = assert!(usize::BITS >= 64, "Token must fit a slotmap key (u64)");
fn token_of(key: ClientKey) -> Token {
    Token(key.data().as_ffi() as usize)
}
fn key_of(token: Token) -> ClientKey {
    KeyData::from_ffi(token.0 as u64).into()
}

struct Client<C> {
    conn: Connection,
    state: C,
}

pub struct App<S: Server> {
    poll: Poll,
    clients: SlotMap<ClientKey, Client<S::ConnState>>,
    handlers: Handlers<S>,
    app_state: S::AppState,
    queues: Queues,
    _listener: TcpListener,
}

impl<S: Server> App<S> {
    const LISTENER: Token = Token(std::usize::MAX);
    const READ_BUDGET: usize = 16;

    pub fn new(
        port: String,
        handlers: Handlers<S>,
        app_state: S::AppState,
    ) -> Result<Self, Box<dyn Error>> {
        let mut listener = TcpListener::bind(format!("127.0.0.1:{port}").parse()?)?;

        let poll = Poll::new()?;

        poll.registry()
            .register(&mut listener, Self::LISTENER, Interest::READABLE)?;

        Ok(App {
            poll,
            clients: SlotMap::with_key(),
            handlers,
            app_state,
            queues: Queues::default(),
            _listener: listener,
        })
    }

    pub fn run(&mut self, events: &mut Events) -> io::Result<()> {
        let timeout = if self.queues.has_pending_reads() {
            Some(Duration::ZERO)
        } else {
            None
        };
        self.poll.poll(events, timeout)?;

        let batch = self.queues.take_pending_reads();
        for &token in &batch {
            self.read_client(token);
        }
        self.queues.recycle(batch);

        for event in events.iter() {
            match event.token() {
                Self::LISTENER => self.accept_clients(),
                token => {
                    let Some(client) = self.clients.get_mut(key_of(token)) else {
                        unreachable!("event for unknown client {token:?}");
                    };

                    if event.is_writable() && client.conn.has_buffered_output() {
                        self.queues.write_later(token);
                    }
                    if event.is_readable() && !self.queues.is_read_pending(token) {
                        self.read_client(token);
                    }
                }
            }
        }

        loop {
            self.apply_operations();
            self.write_pending();
            if !self.queues.has_operations() {
                break;
            }
        }
        Ok(())
    }

    fn accept_clients(&mut self) {
        loop {
            match self._listener.accept() {
                Ok((s, _)) => {
                    if let Err(e) = self.handle_new_conn(s) {
                        println!("failed to set up client: {e}");
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    println!("accept failed: {e}");
                    return;
                }
            }
        }
    }

    fn handle_new_conn(&mut self, s: TcpStream) -> io::Result<()> {
        let ip = s.peer_addr()?.to_string();

        let key = self.clients.insert_with_key(|key| Client {
            conn: Connection::new(s, token_of(key), ip),
            state: S::ConnState::default(),
        });
        let client = &mut self.clients[key];

        if let Err(e) = client.conn.register(
            self.poll.registry(),
            Interest::READABLE | Interest::WRITABLE,
        ) {
            self.clients.remove(key);
            return Err(e);
        }

        (self.handlers.on_connect)(&mut Context {
            state: &mut client.state,
            app: &mut self.app_state,
            conn: &mut client.conn,
            queues: &mut self.queues,
        });

        Ok(())
    }

    fn read_client(&mut self, token: Token) {
        let Some(Client { conn, state }) = self.clients.get_mut(key_of(token)) else {
            return;
        };

        let on_message = self.handlers.on_message;
        let app = &mut self.app_state;
        let queues = &mut self.queues;
        let result = conn.read_lines(Self::READ_BUDGET, |conn, line| {
            on_message(
                &mut Context {
                    state: &mut *state,
                    app: &mut *app,
                    conn,
                    queues: &mut *queues,
                },
                line,
            );
        });

        match result {
            Ok(ReadStatus::Drained) => {}
            Ok(ReadStatus::MoreToRead) => {
                self.queues.read_later(token);
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::UnexpectedEof {
                    println!("read from {token:?} failed: {e}");
                }
                self.queues.push(Operation::Disconnect(token));
            }
        }
    }

    fn write_pending(&mut self) {
        let batch = self.queues.take_pending_writes();
        for &token in &batch {
            let Some(client) = self.clients.get_mut(key_of(token)) else {
                continue;
            };
            if let Err(e) = client.conn.drain_outgoing_messages() {
                println!("write to {token:?} failed: {e}");
                self.queues.push(Operation::Disconnect(token));
            }
        }
        self.queues.recycle(batch);
    }

    fn queue_to(&mut self, token: Token, message: &str) {
        if let Some(client) = self.clients.get_mut(key_of(token)) {
            self.queues.send(&mut client.conn, message);
        }
    }

    fn apply_operations(&mut self) {
        while self.queues.has_operations() {
            let operations = self.queues.take_operations();

            for operation in operations {
                match operation {
                    Operation::Broadcast(message) => {
                        for (_, client) in self.clients.iter_mut() {
                            self.queues.send(&mut client.conn, &message);
                        }
                    }
                    Operation::BroadcastExcl(message, exclude) => {
                        for (_, client) in self.clients.iter_mut() {
                            if exclude.contains(&client.conn.token()) {
                                continue;
                            }
                            self.queues.send(&mut client.conn, &message);
                        }
                    }
                    Operation::SendTo(to, message) => self.queue_to(to, &message),
                    Operation::Disconnect(who) => {
                        if let Some(mut client) = self.clients.remove(key_of(who)) {
                            let _ = client.conn.deregister(self.poll.registry());
                            (self.handlers.on_disconnect)(&mut Context {
                                state: &mut client.state,
                                app: &mut self.app_state,
                                conn: &mut client.conn,
                                queues: &mut self.queues,
                            });
                        }
                    }
                }
            }
        }
    }
}
