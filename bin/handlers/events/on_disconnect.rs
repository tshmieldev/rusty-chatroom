use czat::Context;

use crate::handlers::commands::auth::logout;
use crate::state::Chat;

pub fn on_disconnect(ctx: &mut Context<Chat>) {
    logout(ctx);
}
