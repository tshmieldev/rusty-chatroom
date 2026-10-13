use std::borrow::Cow;

use czat::Context;

use crate::{
    handlers::commands::{self, Command, Commands},
    state::Chat,
};

pub const HELP: Command = Command {
    name: "help",
    description: "shows this menu.",
    run: handle_help,
};

// todo - /help <command>? description and long description?
pub fn handle_help<'a>(ctx: &mut Context<Chat>, _: &'a str) -> Result<(), Option<Cow<'a, str>>> {
    let mut msg = String::with_capacity(256);
    msg += ":::Rusty Chatroom:::\n";
    for &command in ctx.app.commands.by_name.values() {
        msg += &format!("/{}: {}\n", command.name, command.description);
    }
    ctx.send_message(&msg);
    Ok(())
}
