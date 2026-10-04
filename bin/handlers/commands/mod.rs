pub mod auth;
pub mod login;
pub mod logout;
pub mod msg;
pub mod register;
pub mod users;
mod util;
pub mod whoami;

pub use login::LOGIN;
pub use logout::LOGOUT;
pub use msg::MSG;
pub use register::REGISTER;
pub use users::USERS;
pub use whoami::WHOAMI;

use std::borrow::Cow;
use std::cmp::min;
use std::collections::BTreeMap;

use czat::Context;

use crate::state::Chat;

pub type CommandFn = for<'a> fn(&mut Context<Chat>, &'a str) -> Result<(), Option<Cow<'a, str>>>;

#[derive(Clone, Copy)]
pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    run: CommandFn,
}

#[derive(Default)]
pub struct Commands {
    by_name: BTreeMap<&'static str, Command>,
}

impl Commands {
    pub fn register_command(&mut self, command: Command) {
        self.by_name.insert(command.name, command);
    }
    pub fn get(&self, name: &str) -> Option<CommandFn> {
        self.by_name.get(name).map(|command| command.run)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.by_name.values()
    }
}

pub fn handle_command(ctx: &mut Context<Chat>, message: &str) {
    let space_idx = message.find(' ').unwrap_or(message.len());
    let cmdname: &str = &message[1..space_idx];
    let cmd = ctx.app.commands.get(cmdname);
    if cmd.is_none() {
        ctx.send_message("Unknown command. Get some /help");
        return;
    }
    if let Err(oe) = (cmd.unwrap())(ctx, &message[min(space_idx + 1, message.len())..]) {
        match oe {
            Some(e) => ctx.send_message(&e),
            None => ctx.send_message("Invalid usage. Get some /help."),
        }
    };
}
