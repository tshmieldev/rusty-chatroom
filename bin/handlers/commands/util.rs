use std::borrow::Cow;

use czat::Context;

use crate::handlers::filters::{both, except, in_group, logged_in};
use crate::state::{AccountId, Chat, GroupId};

pub fn require_auth<'a>(ctx: &Context<Chat>) -> Result<AccountId, Option<Cow<'a, str>>> {
    ctx.state
        .account()
        .ok_or(Some("You must be logged in.".into()))
}

pub fn broadcast_to_others(ctx: &mut Context<Chat>, message: impl Into<Cow<'static, str>>) {
    let me = ctx.token();
    ctx.broadcast_where(message, both(except(me), logged_in));
}

pub fn broadcast_to_others_in_group(
    ctx: &mut Context<Chat>,
    group_id: GroupId,
    message: impl Into<Cow<'static, str>>,
) {
    let me = ctx.token();
    let members = ctx
        .app
        .group(group_id)
        .map(|g| g.users().collect())
        .unwrap_or_default();
    ctx.broadcast_where(message, both(except(me), in_group(members)));
}
