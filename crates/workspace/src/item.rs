use gpui_kit::{Context, EventEmitter, Focusable, Render, Task};

use gpui_kit::component::{Dock};

pub trait Item: Focusable + EventEmitter<Self::Event> + Render + Sized {
    type Event;

    fn placeholder_text(&self) -> Option<&str> {
        None
    }
    fn on_search(&mut self, _query: &str, cx: &mut Context<Self>);

    fn show_actions(&self) -> bool {
        true
    }

    fn on_close(&mut self, cx: &mut Context<Self>) -> Task<()>;
}
