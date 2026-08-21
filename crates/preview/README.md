## Overview

The preview system activates when a command palette search yields no direct matches. In this case, the system enters preview matching mode, where each registered preview delegate computes a relevance score via the `score_match` method. The result with the highest score is then displayed, with its appearance fully customizable through the delegate's `render_match` implementation.

## Examples

```rust

pub fn init(cx: &mut App) {
    PreviewRegistry::global(cx).update(cx, |registry, cx| {
        registry.register_preview(|cx| cx.new(|_cx| CalculatorPreviewDelegate::new()), cx);
    })
}

pub struct CalculatorPreviewDelegate;

impl CalculatorPreviewDelegate {
    fn new() -> Self {
        Self {}
    }
}

impl PreviewDelegate for CalculatorPreviewDelegate {

    /// Computes a relevance score for the current query.
    fn score_match(&mut self, window: &mut Window, query: &str, cx: &mut Context<Self>) -> Option<u16> {
        // Implement scoring logic based on your use case
        Some(803)
    }

    /// Returns the header text displayed in the preview panel.
    fn header_text(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Arc<str> {
        "Calculator".to_string()
    }

     /// Renders the custom UI for the matched preview item.
    fn render_match(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }

    /// Called when the user confirms/executes this preview item.
    fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        todo!()
    }
}

```