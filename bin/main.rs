mod handlers;
mod state;

use czat::{App, Handlers};
use mio::Events;
use std::error::Error;

use crate::handlers::commands::{LOGIN, LOGOUT, MSG, REGISTER, USERS, WHOAMI};
use crate::handlers::events::{
    on_connect::on_connect, on_disconnect::on_disconnect, on_message::on_message,
};
use crate::state::{AppState, Chat};

fn main() -> Result<(), Box<dyn Error>> {
    let mut app_state = AppState::default();

    app_state.groups.create("logged_in");

    let commands = &mut app_state.commands;
    commands.register_command(REGISTER);
    commands.register_command(LOGIN);
    commands.register_command(LOGOUT);
    commands.register_command(USERS);
    commands.register_command(WHOAMI);
    commands.register_command(MSG);

    let mut app: App<Chat> = App::new(
        String::from("2137"),
        Handlers {
            on_connect,
            on_message,
            on_disconnect,
        },
        app_state,
    )?;

    let mut events = Events::with_capacity(1024);

    loop {
        app.run(&mut events)?;
    }
}
