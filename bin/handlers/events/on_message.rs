use std::collections::HashSet;

use czat::Context;

use crate::handlers::commands::handle_command;
use crate::state::Chat;

pub fn on_message(ctx: &mut Context<Chat>, message: &str) {
    if message.len() == 0 {
        return;
    }
    if message.chars().next().unwrap() == '/' {
        handle_command(ctx, message);
        return;
    }

    if ctx.state.account().is_none() {
        ctx.send_message("You must be logged in to chat.");
    }

    if let Some(id) = ctx.state.account() {
        let user = ctx.app.accounts.get_mut(id).expect("User must exist");

        user.history.push(String::from(message));

        let msg = format!("[{}]: {}", user.username(), message);
        ctx.broadcast_excl(msg, HashSet::from([ctx.token()]));
    }
}
