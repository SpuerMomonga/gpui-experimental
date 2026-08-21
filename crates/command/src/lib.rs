use gpui::{App, AppContext, Entity, Global};


pub fn init(cx: &mut App) {
    let register = cx.new(|_cx| CommandRegistry::new());
    cx.set_global(GlobalCommandRegistry(register));
}

pub struct CommandRegistry {
}

impl CommandRegistry {
    fn new() -> Self {
        Self { }
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalCommandRegistry>().0.clone()
    }
}

struct GlobalCommandRegistry(Entity<CommandRegistry>);

impl Global for GlobalCommandRegistry {}