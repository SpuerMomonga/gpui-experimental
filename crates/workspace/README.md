# workspace

## Overview

`workspace` is the default application page shell. It combines the active view, search,
and actions into a GPUI root view. It does not own feature-specific data or implement the
`command`, `preview`, or `view` registries; those crates provide that state, while
`workspace` responds to changes through entity subscriptions and events.

The Rust code in this crate is currently empty. The sections below describe the boundary
for the first implementation. Add `gpui`, `command`, `preview`, and `view` dependencies
as the implementation requires them; do not pull them in indirectly through the workspace
root package.

## Page Structure

The default workspace has three regions:

1. **Top search bar.** The query input is on the left, with its placeholder supplied by the
   active `ViewDelegate`. The optional `ViewDelegate::render_search_control` area is on the
   right. The view provides its element, while `workspace` applies height and maximum-width
   constraints.
2. **Content area.** Render the active view's GPUI `Entity`/`AnyView`. `workspace` must not
   copy the view's state or render tree.
3. **Bottom action bar.** Show the view title and optional icon on the left, and
   `ViewAction` buttons on the right. Buttons carry GPUI `Action` values and may open an
   action menu or dispatch directly. At most one primary action is shown alongside the bar
   on the left.

The layout itself implements GPUI's `Render` trait, for example with `Workspace` as the
window root entity. Do not make a `ViewDelegate` the window root directly unless it also
implements GPUI's `Render`.

## State and Events

`workspace` should observe at least the following state:

- `CommandRegistry` visibility, enabled state, and registration/removal events, to refresh
  search results;
- `PreviewRegistry` candidate changes, to show a preview when there is no direct match;
- `ViewStore` `Mounted`, `Changed`, and `Unmounted` events, to replace the content area and
  update the action bar.

GPUI already provides `Entity::update`, `Context::observe`, `Context::subscribe`, and
`Context::notify`. These APIs are sufficient for fine-grained updates; do not add a second
global event bus:

```rust
struct Workspace {
    view_subscription: gpui::Subscription,
    query: String,
    selected: Option<usize>,
    active_view: Option<view::ViewId>,
}

impl Workspace {
    fn new(cx: &mut gpui::Context<Self>) -> Self {
        let view_store = view::ViewStore::global(cx);
        let view_subscription = cx.subscribe(&view_store, |workspace, _, event, cx| {
            workspace.apply_view_event(event, cx);
        });
        Self {
            view_subscription,
            query: String::new(),
            selected: None,
            active_view: None,
        }
    }
}
```

Event sources implement `EventEmitter<Event>`; `workspace` is a subscriber and does not
need to implement the same event interface. `workspace` must retain each `Subscription`
for its lifetime; dropping the return value in `new` causes GPUI to cancel the subscription
immediately.

## Search and View Synchronization

On every query change, update `workspace`'s own `query`, then pass the active `View` and
the same string to `ViewDelegate::perform_search`. The callback runs for every input change and
receives an empty string when the input is cleared. A view may update filter state, start
an asynchronous search, call `view.remove_view()`, and refresh its entity with its own
`Context::notify()`. The search bar's `render_search_control` also receives the active `View`, but
only renders supplemental controls; it does not read or cache the query string.

```text
query changed
      |
      +---- active view -> ViewDelegate::perform_search(view, query)
      |
      +---- visible + enabled + parameterless commands ---- match ----> command list
      |
      +---- no direct match ----> preview providers -> best score -> preview body
```

The command palette can pass only command IDs, so commands with custom argument fields do
not appear in command results; a view search callback does not change this restriction.
Confirming a command calls `CommandRegistry::execute`; confirming a preview calls the
selected provider's `confirm`. Both run on the GPUI application thread and may request
`view.remove_view()` through the `&mut View` received by the callback.

## Actions

View actions use GPUI's `Action` trait and are declared with `actions!` or
`#[derive(Action)]`. `ViewAction` adds display metadata such as a title and icon, and keeps
`Box<dyn Action>` internally so a list can contain heterogeneous actions. Buttons call
`window.dispatch_action(action.action().boxed_clone(), cx)`. Keyboard shortcuts and mouse
clicks are handled by the same GPUI action listeners; `workspace` does not parse action
names or maintain a second command-ID protocol.

## Window Lifecycle

`workspace` itself can be created with `App::open_window`. `WindowHandle<Workspace>` does not
keep the window alive; after the window closes, GPUI's `App::on_window_closed` callback
notifies `workspace` to clear its window state. Do not treat a `WindowId` as a logical
`ViewId`, and do not access an invalidated `Window` from a window-close callback.

The view's hosting mode is selected by `ViewOptions`: `ViewOptions::View` uses only the
workspace, `ViewOptions::Detachable` additionally stores `WindowOptions` for a detached
window, and `ViewOptions::Window` uses GPUI's window configuration directly. The `view`
crate owns `App::open_window`, stores the `AnyWindowHandle`, and emits `Unmounted` from the
close callback; `workspace` only displays logical view state.

## Implementation Order

1. Implement the `Workspace` `Render` root entity and render the static three-region layout.
2. Connect the active view from `ViewStore` and GPUI's `AnyView`; verify that
   `Context::notify()` invalidates only the content area.
3. Connect command search, then add preview fallback, `search` synchronization, and the
   confirmation flow.
4. Add the GPUI actions menu, primary action, window-close cleanup, and multi-window tests.

Once the workspace crate has real `Cargo.toml` dependencies, update the root
`ARCHITECTURE.md` and this README together. Do not introduce a new shared UI crate merely
to illustrate the design.
