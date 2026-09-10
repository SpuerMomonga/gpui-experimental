use gpui_kit::{Context, IntoElement, Render, Window, div};

pub struct TitleBar {}

impl TitleBar {
    pub fn new() -> Self {
        Self {}
    }
}

impl Render for TitleBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}