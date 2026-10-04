use std::borrow::Cow;

use czat::Context;

use super::auth::{create_account, login};
use super::Command;
use crate::state::Chat;

pub const REGISTER: Command = Command {
    name: "register",
    description: "<username> <password> <password>: create an account and log in",
    run: handle_register,
};

pub fn handle_register<'a>(
    ctx: &mut Context<Chat>,
    message: &'a str,
) -> Result<(), Option<Cow<'a, str>>> {
    if message.matches(' ').count() != 2 {
        return Err(None);
    };

    if ctx.state.account().is_some() {
        return Err(Some("You are already logged in.".into()));
    }

    let args = &mut message.split(' ').into_iter();

    let username = args.next().unwrap();
    let password = args.next().unwrap();
    let repeat_password = args.next().unwrap();

    if ctx.app.accounts.find(username).is_some() {
        return Err(Some("Username taken.".into()));
    }
    if password != repeat_password {
        return Err(Some("Passwords don't match.".into()));
    }

    let id = create_account(ctx, username, password)?;
    login(ctx, id);

    Ok(())
}
