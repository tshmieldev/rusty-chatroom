use std::borrow::Cow;
use std::collections::HashSet;

use mio::Token;

pub(crate) enum Operation {
    Broadcast(Cow<'static, str>),
    BroadcastExcl(Cow<'static, str>, HashSet<Token>),
    SendTo(Token, Cow<'static, str>),
    Disconnect(Token),
}
