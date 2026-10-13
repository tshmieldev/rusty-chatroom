use std::collections::BTreeSet;

use mio::Token;

use crate::state::{AccountId, ConnectionState};

pub fn logged_in(_: Token, state: &ConnectionState) -> bool {
    state.account().is_some()
}

pub fn except(token: Token) -> impl Fn(Token, &ConnectionState) -> bool {
    move |t, _| t != token
}

pub fn in_group(members: BTreeSet<AccountId>) -> impl Fn(Token, &ConnectionState) -> bool {
    move |_, state| state.account().is_some_and(|id| members.contains(&id))
}

pub fn both(
    a: impl Fn(Token, &ConnectionState) -> bool,
    b: impl Fn(Token, &ConnectionState) -> bool,
) -> impl Fn(Token, &ConnectionState) -> bool {
    move |t, state| a(t, state) && b(t, state)
}
