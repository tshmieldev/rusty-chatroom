use std::borrow::Cow;
use std::collections::HashSet;

use mio::Token;

pub(crate) type Filter<C> = Box<dyn Fn(Token, &C) -> bool>;

pub(crate) enum Operation<C> {
    Broadcast(Cow<'static, str>),
    BroadcastWhere(Cow<'static, str>, Filter<C>),
    BroadcastTo(Cow<'static, str>, HashSet<Token>),
    BroadcastExcl(Cow<'static, str>, HashSet<Token>),
    SendTo(Token, Cow<'static, str>),
    Disconnect(Token),
}
