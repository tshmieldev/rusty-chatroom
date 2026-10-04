use std::borrow::Cow;

use czat::Context;

use super::util::require_auth;
use crate::{handlers::commands::Command, state::Chat};

pub const WHOAMI: Command = Command {
    name: "whoami",
    description: "prints your client info.",
    run: handle_whoami,
};

pub fn handle_whoami<'a>(
    ctx: &mut Context<Chat>,
    message: &'a str,
) -> Result<(), Option<Cow<'a, str>>> {
    let id = require_auth(ctx)?;

    if message.len() > 0 {
        return Err(None);
    }

    let user = ctx.app.accounts.get(id).expect("User must exist.");

    let mut group_string = String::with_capacity(32);

    for (i, &group_id) in user.groups().into_iter().enumerate() {
        group_string += ctx.app.group(group_id).expect("Group must exist.").name();
        if i != user.groups().len() - 1 {
            group_string += ", ";
        }
    }

    let mut msg = String::with_capacity(256);
    msg.push_str("------------------------\n");
    msg.push_str(&format!("Username: {}\n", user.username()));
    msg.push_str(&format!("Groups: {}\n", group_string));
    msg.push_str(&format!("Ip: {}\n", ctx.ip()));
    msg.push_str("------------------------\n");

    ctx.send_message(&msg);

    Ok(())
}
