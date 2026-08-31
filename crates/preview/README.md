# preview

## Overview

When command search has no direct result, `command-palette` can delegate the query to
`preview`. Each preview provider returns a comparable score; the highest-scoring candidate
renders the preview and executes the operation when the user confirms it.

This crate does not yet implement a registry. The API below is a GPUI-aligned proposal:
preview providers should be GPUI `Entity<T>` values rendered by the host through an adapter,
rather than making GPUI understand an additional rendering protocol.

The current `Cargo.toml` has no dependencies. Implementing this proposal will require at
least `gpui` and `anyhow`; add them when `PreviewRegistry` is implemented instead of
expanding the dependency graph solely for documentation examples.

## Design Notes

`score_match` computes how well a query matches the current application state. If a query
needs asynchronous data, the provider can update itself through `Context::spawn` and then
call `cx.notify()` to refresh results; rendering callbacks should read current state and
must not block while waiting for data.

When multiple providers receive the same score, the registry uses stable `priority` and
registration order to select a result; it must not depend on hash-map iteration order. A
provider returns `None` when the query does not match, and the host renders no preview area
when there is no candidate.

`confirm` runs on the GPUI application thread and may call `command` or `view`. Providers
usually use the `&mut Window` supplied for the call to perform window-specific work, but the
registry must not store a window borrow or another non-sendable temporary reference because
the same provider may serve multiple windows.

## Proposed API

```rust
use gpui::{Context, Entity, IntoElement, SharedString, Window};

pub trait PreviewDelegate: 'static + Sized {
    /// Returns a score for this query. `None` means no match.
    fn score_match(&mut self, query: &str, cx: &mut Context<Self>) -> Option<u16>;

    /// Text shown above the preview body.
    fn header_text(&self) -> SharedString;

    /// Renders the body of the preview.
    fn render_match(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement;

    /// Confirms the currently previewed result.
    fn confirm(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()>;
}

pub struct PreviewRegistry {
    // provider IDs, priorities, and provider factories
}

pub type PreviewId = u64;

pub struct PreviewRegistration {
    // private provider ID and generation
}

impl PreviewRegistry {
    pub fn global(cx: &gpui::App) -> Entity<Self>;

    pub fn register<T>(
        &mut self,
        priority: i32,
        build: impl FnOnce(&mut gpui::App) -> Entity<T> + 'static,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<PreviewRegistration>
    where
        T: PreviewDelegate;

    pub fn best_match(
        &mut self,
        query: &str,
        window: &mut Window,
        cx: &mut gpui::App,
    ) -> Option<PreviewMatch>;
}

pub struct PreviewMatch {
    pub provider_id: PreviewId,
    pub score: u16,
    pub view: gpui::AnyView,
}
```

If `PreviewDelegate` cannot implement GPUI's `Render`, `PreviewRoot<T>` should forward
`render_match` and then convert `Entity<PreviewRoot<T>>` to `AnyView`. Keep the delegate's
rendering method as `impl IntoElement`, so each provider can return its own GPUI element
type. Call `into_any_element()` only at an adapter boundary that combines providers in a
shared preview result or element tree. The caller-facing trait therefore does not need to
return `AnyElement` merely for registry type erasure.

## Example

```rust
impl PreviewDelegate for CalculatorPreview {
    fn score_match(&mut self, query: &str, _cx: &mut gpui::Context<Self>) -> Option<u16> {
        query.strip_prefix("=").map(|_| 800)
    }

    fn header_text(&self) -> gpui::SharedString {
        "Calculator".into()
    }

    fn render_match(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        gpui::div()
    }

    fn confirm(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> anyhow::Result<()> {
        // Commit the calculation or dispatch a command here.
        Ok(())
    }
}
```
