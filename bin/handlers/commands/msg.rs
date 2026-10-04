use std::borrow::Cow;

use czat::Context;

use super::util::require_auth;
use crate::{handlers::commands::Command, state::Chat};

pub const MSG: Command = Command {
    name: "msg",
    description: "<user> <message>: sends the [message] only to [user].",
    run: handle_msg,
};

pub fn handle_msg<'a>(
    ctx: &mut Context<Chat>,
    message: &'a str,
) -> Result<(), Option<Cow<'a, str>>> {
    let uid = require_auth(ctx)?;

    let Some(wsi) = message.find(" ") else {
        return Err(None);
    };

    let sender = ctx.app.user(uid).expect("User must exist.");

    let recipient_name = &message[..wsi];

    let Some(user_id) = ctx.app.user_by_name(recipient_name) else {
        return Err(Some("User doesn't exist.".into()));
    };

    let recipient_token = ctx.app.session(user_id).expect("User is not online.");

    let mut msg = String::with_capacity(256);
    msg.push_str(&format!(
        "{} whispers: {}",
        sender.username(),
        message[wsi..].trim()
    ));

    ctx.send_to(recipient_token, msg);

    Ok(())
}
