use czat::Context;

use crate::state::Chat;

pub fn on_connect(ctx: &mut Context<Chat>) {
    println!("New connection from {:}", ctx.ip());
    ctx.send_message("Welcome to Chat!");
    ctx.send_message("/login or /register\n/help for details");
}
