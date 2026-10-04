use std::borrow::Cow;

use czat::Context;

use crate::state::{AccountId, Chat};

pub fn require_auth<'a>(ctx: &Context<Chat>) -> Result<AccountId, Option<Cow<'a, str>>> {
    ctx.state
        .account()
        .ok_or(Some("You must be logged in.".into()))
}
