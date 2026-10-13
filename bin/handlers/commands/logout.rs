use std::borrow::Cow;

use czat::Context;

use super::Command;
use super::auth::logout;
use super::util::require_auth;
use crate::state::Chat;

pub const LOGOUT: Command = Command {
    name: "logout",
    description: "log out",
    run: handle_logout,
};

pub fn handle_logout<'a>(ctx: &mut Context<Chat>, _: &'a str) -> Result<(), Option<Cow<'a, str>>> {
    let user = require_auth(ctx)?;
    ctx.broadcast(format!(
        "{} has left.",
        ctx.app.user(user).unwrap().username()
    ));
    logout(ctx)
}
