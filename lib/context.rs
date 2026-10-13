use std::borrow::Cow;
use std::collections::HashSet;

use mio::Token;

use crate::queues::Queues;
use crate::{connection::Connection, operation::Operation, server::Server};

pub struct Context<'a, S: Server> {
    pub state: &'a mut S::ConnState,
    pub app: &'a mut S::AppState,
    pub(crate) conn: &'a mut Connection,
    pub(crate) queues: &'a mut Queues<S::ConnState>,
}

impl<'a, S: Server> Context<'a, S> {
    pub fn ip(&self) -> &str {
        self.conn.ip()
    }
    pub fn token(&self) -> Token {
        self.conn.token()
    }
    pub fn send_message(&mut self, message: &str) {
        self.queues.send(self.conn, message);
    }
    pub fn broadcast(&mut self, message: impl Into<Cow<'static, str>>) {
        self.queues.push(Operation::Broadcast(message.into()));
    }
    pub fn broadcast_to(&mut self, message: impl Into<Cow<'static, str>>, incl: HashSet<Token>) {
        self.queues
            .push(Operation::BroadcastTo(message.into(), incl));
    }
    pub fn broadcast_excl(&mut self, message: impl Into<Cow<'static, str>>, excl: HashSet<Token>) {
        self.queues
            .push(Operation::BroadcastExcl(message.into(), excl));
    }
    pub fn broadcast_where(
        &mut self,
        message: impl Into<Cow<'static, str>>,
        filter: impl Fn(Token, &S::ConnState) -> bool + 'static,
    ) {
        self.queues
            .push(Operation::BroadcastWhere(message.into(), Box::new(filter)));
    }
    pub fn send_to(&mut self, to: Token, message: impl Into<Cow<'static, str>>) {
        self.queues.push(Operation::SendTo(to, message.into()));
    }
    pub fn disconnect(&mut self, who: Token) {
        self.queues.push(Operation::Disconnect(who));
    }
}
