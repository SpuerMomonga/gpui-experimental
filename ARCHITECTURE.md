# Architecture

## Scope

`gpui-experimental` is a small set of composable crates for an application that is
primarily driven by commands and views. The repository is a design and implementation
scaffold, not a finished framework: public APIs in the crate READMEs are proposals until
the corresponding Rust code exists and is covered by tests.

The design follows GPUI's ownership model:

```text
App / Global stores
        |
        +-- command registry ---- command-palette
        |
        +-- preview registry
        |
        +-- view store ---------- workspace
        |
        +-- GPUI windows and Render roots
```

`App` owns application state on the main thread. Registries are GPUI entities stored in
typed globals. A view is a GPUI `Entity<T>` whose `T` implements `Render`, or a documented
adapter that turns a feature-specific delegate into such an entity. State changes use
GPUI's `notify`/`observe`; semantic changes use typed `EventEmitter` events and
`subscribe`.

## Crate Responsibilities

| Crate | Responsibility | Must not own |
| --- | --- | --- |
| `collections` | Shared collection aliases and re-exports used across crates. | UI state, registries, or application policy. |
| `command` | Application-level command metadata, registration, lookup, execution, and command lifecycle events. Commands must work without a window. | Search-result layout, window creation, or a second keybinding system. |
| `macros` | Shared proc-macro implementations, including command argument schema/decoding derives. | Command storage, registration, or execution policy. |
| `preview` | Preview providers used when a query has no direct command match: scoring, rendering, and confirmation. | Command registration, global window policy, or palette layout. |
| `view` | The view lifecycle and identity layer: mounting and unmounting feature views, tracking their GPUI entities, and exposing typed/erased handles. | The workspace's three-region layout or command-palette presentation. |
| `workspace` | The default host window and its composition of search/header, view body, and actions footer. It observes command, preview, and view state. | Feature-specific view state or registry implementation. |
| `command-palette` | The default palette UI that queries `command` and falls back to `preview`. | Owning command definitions or replacing GPUI's action/keymap system. |

`command-palette` is intentionally separate from `workspace`: the palette can be used as
the workspace header or hosted elsewhere, while `workspace` remains the default page
shell.

## Lifecycle and Data Flow

1. Application startup calls each crate's `init(&mut App)` once. Each initializer creates
   its registry with `cx.new` and stores it behind a private `Global` newtype.
2. Feature modules register commands and preview providers during startup or plugin
   activation. Command registration is intentionally simple: the first command with an
   ID wins, later duplicates are ignored, and removal is explicit through `unregister`.
   Each handler's parameter type provides a macro-generated schema with names, kinds, defaults,
   and descriptions; the registry decodes and validates an input before calling its handler.
3. A command-palette query first filters enabled, effectively visible, parameterless
   commands (`Schema::fields.is_empty()`). A command with custom argument fields is
   not eligible for palette exposure because
   the palette can invoke only a command ID and cannot collect those arguments. If there is
   no direct result, it asks preview providers for scores. The highest score is rendered;
   ties are resolved by registration order or an explicit provider priority.
4. Executing a command or confirming a preview runs on the GPUI application thread. A
   handler may update entities, mount a view, or call `App::open_window`, but the registry
   itself must not require a `Window`.
5. `ViewStore` mounts a view entity and emits `Mounted`, `Changed`, and `Unmounted` events.
   `workspace`
   subscribes to those events and renders the active view entity as part of its root
   `Render` implementation. A detached view is represented by a GPUI window handle and
   uses GPUI's normal window-close callback.
6. Each workspace query change is forwarded to the active `ViewDelegate::perform_search` callback
   before command and preview results are rendered. The `View`/`ViewStore` lifecycle controller
   owns mount and unmount operations; `View::remove_view()` records a removal request that the
   store applies after the control operation returns. The `ViewContext::open_view` build closure
   receives the controller for lifecycle registration, and every ordinary `ViewDelegate` callback
   receives the current `&mut View` for removal requests and view-scoped state. `ViewDelegate`
   does not declare separate `mount` or `unmount` methods. The `View::on_mount` and
   `View::on_unmount` hooks receive the current `&mut App` when the store dispatches them.
   View state changes call `Context::notify()`;
   workspace-owned subscriptions should
   invalidate only the affected portion of the page; broad global refreshes are a fallback,
   not the normal update path.

## GPUI-Compatible API Decisions

### Use `Render` at the window boundary

GPUI's `App::open_window` requires `V: Render` and calls
`FnOnce(&mut Window, &mut App) -> Entity<V>`. A proposed `ViewDelegate::render_view`
method is useful as a feature contract, but it cannot be passed directly to
`open_window`. The implementation should therefore use one of these two approaches:

- Prefer making the feature delegate itself implement `Render` and keep the API surface
  identical to GPUI.
- If the delegate must expose extra lifecycle methods, wrap it in a small `ViewRoot<V>`
  adapter that implements `Render` by forwarding to `V::render_view`. The adapter owns
  the lifecycle subscriptions and stores the original typed `Entity<V>`.

`ViewDelegate::render_view` and preview rendering methods should return `impl IntoElement`.
The concrete delegate is already held behind a typed `Entity<V>`, so callers should keep their
element type generic. Convert to an erased element only inside an adapter that genuinely needs
to combine heterogeneous elements; the public delegate API should not require `AnyElement`.

The adapter should not duplicate `Entity`, `AnyView`, or `WindowHandle` behavior. Use
GPUI's `AnyView`/`AnyEntity` for type erasure and keep the actual
`WindowHandle<ViewRoot<V>>` at the creation boundary. A public `ViewHandle` may expose its
`AnyWindowHandle` after that conversion. The `View` controller must not store a `Window` value,
because GPUI exposes `Window` only as a temporary mutable borrow. The view layer must not invent
a second window ID.

### Separate application and window handles

`ViewId` identifies one mount operation and is stable for the lifetime of its store entry.
`WindowHandle<V>` identifies a concrete GPUI window and becomes invalid when the window closes.
A logical view may be rendered in the workspace, in a detached window, or in neither; these are
different states and must not be conflated. View identity is not derived from a separate feature
key: callers retain a `ViewHandle` when they need to address one mounted instance.

### Prefer typed events over a custom event bus

Registry and view events should be plain Rust event values, for example a `CommandEvent` enum
with `Registered`, `Changed`, and `Removed` variants, alongside `ViewMounted`, `ViewChanged`, and `ViewUnmounted`, with
the emitting entity implementing `EventEmitter<Event>`. Consumers keep the returned GPUI
`Subscription`. Non-visual registries do not need a second `notify()` for the same lifecycle
event; visual consumer entities call `Context::notify()` when their own render state changes.

Public registration methods should accept `impl Fn` directly. A registry may box a
callback internally to store heterogeneous handlers, but that type-erasure detail must not
leak into the caller-facing API.

The `command` crate also exposes a `CommandContext` extension trait for `gpui::App`. Its
`register`, `unregister`, `execute`, and `update` methods forward to the initialized global
registry, while the entity-level methods remain available for code that already has a registry
handle.

Command metadata is non-generic. The handler closure determines its input type, similar to
Bevy's `IntoSystem`: a user-defined argument struct derives `Arg` and receives decoded external
input, while a no-argument command receives the flat host-owned `HostArgs` struct. The
registry boundary has exactly two variants, `Input::External(Value)` and
`Input::Internal(HostArgs)`. The palette accepts only commands whose generated schema is
empty; internal fields are accessed directly, not through string-keyed maps or nested source
enums.

### Use GPUI actions for keybindings

Command metadata may display a shortcut, but command execution should not parse a string
and bypass GPUI. View actions use the same `Action` trait: `ViewAction` stores a
`Box<dyn Action>` internally only to hold heterogeneous action types, while its constructor
accepts a concrete action directly. The box remains private to `ViewAction`; buttons dispatch
`boxed_clone()` through the current
`Window`, and handlers registered with `Window::on_action`, element `.on_action`, or
`App::on_action` receive the original concrete type. When a command needs a keybinding, define
a GPUI `Action`, register it in the application keymap, and have its handler call the command
registry. `Keystroke` and `KeyBinding` already provide parsing and context-aware precedence.

## Open Design Questions

- The handler inference trait and `Arg` derive macro are still proposals. Keep the
  generated runtime schema small and JSON-compatible for CLI/IPC discovery, while making the
  handler receive its declared typed parameter. Keep the two input variants explicit and do not
  add a generic parameter to `Command` merely to carry schema information.
- `ViewOptions` keeps the three placement forms used by the view API: `View(ViewConfig)` for
  workspace-only views, `Detachable(DetachableConfig)` for views with independent-window
  configuration, and `Window(WindowOptions)` for a direct GPUI window. `WindowOptions` remains
  a window-creation value and is passed only to `App::open_window`.
- IPC and CLI dispatch are mentioned by the command design but have no crate or runtime
  implementation yet. Add that integration only after choosing a transport and defining
  request/response and error semantics.
- Persistence of view visibility and detached-window bounds is outside the current
  crates. It should be added as an application service rather than hidden in `ViewStore`.

## Change Policy

When crate ownership, public API status, or lifecycle boundaries change, update this file
with the implementation in the same change. Keep diagrams and examples intentionally
small; the code and the locked GPUI revision are the source of truth for details.
