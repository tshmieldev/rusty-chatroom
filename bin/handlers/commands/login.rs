use std::borrow::Cow;

use czat::Context;

use super::Command;
use super::auth::login;
use crate::state::Chat;

pub const LOGIN: Command = Command {
    name: "login",
    description: "<username> <password>: log in",
    run: handle_login,
};

pub fn handle_login<'a>(
    ctx: &mut Context<Chat>,
    message: &'a str,
) -> Result<(), Option<Cow<'a, str>>> {
    if message.matches(' ').count() != 1 {
        return Err(None);
    };

    if ctx.state.account().is_some() {
        return Err(Some("You are already logged in.".into()));
    }

    let args = &mut message.split(' ').into_iter();

    let username = args.next().unwrap();
    let password = args.next().unwrap();

    let Some(id) = ctx.app.accounts.find(username) else {
        return Err(Some("Wrong username or password.".into()));
    };
    if !ctx
        .app
        .accounts
        .get(id)
        .expect("User must exist.")
        .check_password(password)
    {
        return Err(Some("Wrong username or password.".into()));
    }

    login(ctx, id)?;

    ctx.send_message("Logged in successfully.");

    Ok(())
}
