use gpui_kit::{Context, IntoElement, Render, Window, div};

pub struct ActionsBar {}

impl ActionsBar {
    pub fn new() -> Self {
        Self {}
    }
}

impl Render for ActionsBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}