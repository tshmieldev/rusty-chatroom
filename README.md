This is a TCP server library and a chatroom server written in rust.

The goal of this project was for me to:
- Get better at Rust
- Have fun with kernel queues in rust (via mio)
- Have fun with raw TCP streams
- See how much easier (and sometimes, harder) making such stuff is, compared to raw C with no libraries

Here are some docs, written by Bruce (thanks Bruce!)

> [!NOTE]
> Content below this message is AI-generated

## Features

- **Line protocol**: incoming data is split on `\n`, and each line reaches your handler as a `&str`.
- **Per-connection and app-wide state**: you pick the types; handlers get both through `Context`.
- **Fair scheduling**: each client gets at most 16 reads (64 KB) per loop iteration, so one busy client can't stall the rest.
- **Disconnect detection**: closed or reset connections are removed, and `on_disconnect` runs.
- **Slow-client protection**: a client more than 1 MB behind on reading is disconnected (IRC's "Max SendQ exceeded").
- **Line length limit**: a line over 4096 bytes disconnects the sender.
- **Stale tokens are safe**: client tokens carry a generation, so a token for a client that has left never reaches a new client that reuses its slot.
- **Batched writes**: all output queued during one loop iteration goes out together.

## Getting started

```toml
[dependencies]
czat = { git = "https://github.com/tshmieldev/rusty-chatroom" }
mio = { version = "1", features = ["net", "os-poll"] }
```

`mio` is needed for `mio::Events` (passed to `App::run`) and `mio::Token` (identifies a client).

### Echo server

The smallest possible server. Handlers you don't set default to doing nothing.

```rust
use czat::{App, Context, Handlers, Server};
use mio::Events;

struct Echo;

impl Server for Echo {
    type ConnState = ();
    type AppState = ();
}

fn on_message(ctx: &mut Context<Echo>, message: &str) {
    ctx.send_message(message);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app: App<Echo> = App::new(
        String::from("7000"),
        Handlers {
            on_message,
            ..Default::default()
        },
        (),
    )?;

    let mut events = Events::with_capacity(1024);
    loop {
        app.run(&mut events)?;
    }
}
```

Try it with `nc 127.0.0.1 7000`.

### Chat room with state

```rust
use std::collections::HashSet;

use czat::{App, Context, Handlers, Server};
use mio::Events;

#[derive(Default)]
struct Session {
    name: Option<String>,
}

struct Lobby {
    messages_sent: u64,
}

struct Chat;

impl Server for Chat {
    type ConnState = Session;
    type AppState = Lobby;
}

fn on_connect(ctx: &mut Context<Chat>) {
    println!("{} connected", ctx.ip());
    ctx.send_message("Welcome! Pick a name with /nick <name>.");
}

fn on_message(ctx: &mut Context<Chat>, message: &str) {
    if let Some(name) = message.strip_prefix("/nick ") {
        ctx.state.name = Some(name.to_string());
        ctx.send_message("Name set.");
        return;
    }

    let name = ctx.state.name.as_deref().unwrap_or("anon");
    let line = format!("[{name}]: {message}");
    ctx.app.messages_sent += 1;

    let me = ctx.token();
    ctx.broadcast_excl(line, HashSet::from([me]));
}

fn on_disconnect(ctx: &mut Context<Chat>) {
    if let Some(name) = ctx.state.name.take() {
        ctx.broadcast(format!("{name} left."));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app: App<Chat> = App::new(
        String::from("2137"),
        Handlers {
            on_connect,
            on_message,
            on_disconnect,
        },
        Lobby { messages_sent: 0 },
    )?;

    let mut events = Events::with_capacity(1024);
    loop {
        app.run(&mut events)?;
    }
}
```

## API

### `Server`

Ties your two state types together:

```rust
impl Server for MyServer {
    type ConnState = MyConnState; // one per client, created with Default::default()
    type AppState = MyAppState;   // one for the whole app, passed to App::new
}
```

### `Handlers`

| Field | Called when |
|---|---|
| `on_connect: fn(&mut Context<S>)` | a client connects |
| `on_message: fn(&mut Context<S>, &str)` | a full line arrives (without the `\n`) |
| `on_disconnect: fn(&mut Context<S>)` | a client is removed: it closed the connection, hit an error or a limit, or you called `ctx.disconnect` |

Handlers are plain `fn` items, not closures. Put shared data in `AppState`.

### `Context`

| Item | Description |
|---|---|
| `ctx.state` | this client's `ConnState` (`&mut`) |
| `ctx.app` | the shared `AppState` (`&mut`) |
| `ctx.ip()` | client address, e.g. `"127.0.0.1:52144"` |
| `ctx.token()` | this client's `mio::Token`; store it to reach the client later |
| `ctx.send_message(&str)` | send a line to this client |
| `ctx.send_to(token, msg)` | send a line to another client |
| `ctx.broadcast(msg)` | send a line to every client |
| `ctx.broadcast_excl(msg, HashSet<Token>)` | send to everyone except the given clients |
| `ctx.disconnect(token)` | disconnect a client (its `on_disconnect` runs) |

`send_to`, `broadcast` and `broadcast_excl` accept a `&'static str` (no allocation) or a `String`. A `\n` is appended to every message.

Nothing is written during the handler. Messages are queued and written at the end of the current loop iteration. `send_to`, `broadcast` and `disconnect` are also applied only after your handler returns.

## The bundled chat server

```sh
cargo run
nc 127.0.0.1 2137
```

| Command | Description |
|---|---|
| `/register <username> <password> <password>` | create an account and log in |
| `/login <username> <password>` | log in |
| `/logout` | log out |
| `/users [group]` | list users in a group (default: everyone logged in) |
| `/whoami` | show your username, groups and IP |
| `/msg <user> <message>` | send a private message |

Any other line is broadcast to everyone, once you're logged in.

## Limits

| Limit | Value |
|---|---|
| Max line length | 4096 bytes |
| Max unsent output per client | 1 MB |
| Reads per client per loop iteration | 16 × 4 KB |
| Platform | 64-bit only (checked at compile time) |
| Threads | 1 |

## License

MIT, see [LICENSE](LICENSE).
