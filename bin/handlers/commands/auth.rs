use std::borrow::Cow;

use czat::Context;

use crate::state::{AccountId, Chat};

pub fn create_account<'a>(
    ctx: &mut Context<Chat>,
    username: &str,
    password: &str,
) -> Result<AccountId, Option<Cow<'a, str>>> {
    ctx.app
        .accounts
        .register(username, password)
        .ok_or(Some("Username taken.".into()))
}

pub fn login<'a>(ctx: &mut Context<Chat>, id: AccountId) -> Result<(), Option<Cow<'a, str>>> {
    let token = ctx.token();
    ctx.app.set_online(ctx.state, id, token);
    let gid = ctx
        .app
        .groups
        .find("logged_in")
        .expect("logged_in group must exist.");
    ctx.app.add_user_to_group(id, gid)?;
    Ok(())
}

pub fn logout<'a>(ctx: &mut Context<Chat>) -> Result<(), Option<Cow<'a, str>>> {
    let uo = ctx.state.account();
    if let Some(user_id) = uo {
        ctx.app.remove_user_from_all_groups(user_id)?;
        ctx.app.set_offline(ctx.state);
    }
    Ok(())
}

pub fn send_to_account(ctx: &mut Context<Chat>, id: AccountId, message: String) -> bool {
    match ctx.app.session(id) {
        Some(token) => {
            ctx.send_to(token, message);
            true
        }
        None => false,
    }
}
