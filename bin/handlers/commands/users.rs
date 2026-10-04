use std::borrow::Cow;

use czat::Context;

use super::Command;
use super::util::require_auth;
use crate::state::Chat;

pub const USERS: Command = Command {
    name: "users",
    description: "[group]: list users in a group (default: everyone logged in)",
    run: handle_users,
};

pub fn handle_users<'a>(
    ctx: &mut Context<Chat>,
    message: &'a str,
) -> Result<(), Option<Cow<'a, str>>> {
    require_auth(ctx)?;

    let trimmed_message = message.trim();
    if trimmed_message.contains(' ') {
        return Err(Some("Group name invalid.".into()));
    };

    let group_name = if !trimmed_message.is_empty() {
        trimmed_message
    } else {
        "logged_in"
    };

    let Some(gid) = ctx.app.groups.find(group_name) else {
        return Err(Some("Group doesn't exist.".into()));
    };

    let g = ctx.app.groups.get(gid).expect("Group must exist.");

    let mut m = String::with_capacity(64);
    if !trimmed_message.is_empty() {
        m += "Users in #";
        m += group_name;
        m += ": "
    } else {
        m += "Active users: "
    }

    for (i, user) in g.users().enumerate() {
        if i > 0 {
            m += ", ";
        }
        m += ctx
            .app
            .accounts
            .get(user)
            .expect("User must exist.")
            .username();
    }

    ctx.send_message(&m);

    Ok(())
}
