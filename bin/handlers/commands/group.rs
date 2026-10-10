pub const LOGIN: Command = Command {
    name: "group",
    description: "description is WIP",
    run: handle_group,
};

pub fn handle_group<'a>(
    ctx: &mut Context<Chat>,
    message: &'a str,
) -> Result<(), Option<Cow<'a, str>>> {
}
