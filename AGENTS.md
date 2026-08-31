# AGENTS.md

## Project Overview

This is an experimental project that builds modular application features on top of
[`gpui`](https://github.com/zed-industries/zed/tree/main/crates/gpui). The workspace is
organized around the `command`, `preview`, `view`, and `workspace` crates, with
`command-palette` providing the default command-search UI. The crates expose small,
composable APIs so that other application modules can contribute commands and views
without depending on the concrete workspace layout.

The application is intentionally designed to be useful before a window exists. A
hotkey, an IPC request, or another application-level event may wake the application and
ask it to create a window later. Code that only needs application state must therefore
not assume that a `Window` is available.

## Architecture

[ARCHITECTURE.md](./ARCHITECTURE.md) is the source of truth for repository structure,
crate responsibilities, ownership boundaries, and the status of proposed APIs. Read it
before adding files or moving behavior.

When a package under `crates` is added, removed, renamed, or its responsibility changes,
update [ARCHITECTURE.md](./ARCHITECTURE.md) in the same change.

## GPUI Integration Rules

- Treat `App` as the main-thread application context. Use `AppContext::new` to create
  entities and GPUI's `Global` mechanism for application-wide stores.
- Treat `Entity<T>` as the ownership and observation boundary. Prefer typed entities,
  `cx.notify()` for render invalidation, and `EventEmitter` plus `cx.emit()` for semantic
  events. Do not introduce a second event bus when GPUI's subscriptions are sufficient.
- Implement visual roots with GPUI's `Render` trait. `App::open_window` returns a
  `WindowHandle<V>` and invokes its builder with both `&mut Window` and `&mut App`.
  `WindowHandle` does not keep a window alive; closing is performed through
  `Window::remove_window`, and application-level close notifications use
  `App::on_window_closed`.
- Treat a view's `mount`/`unmount` lifecycle as separate from GPUI window lifetime. A
  `ViewId` identifies one mounted view, while `WindowId` and `WindowHandle` identify a
  concrete window. Window-close callbacks must feed back into the view's unmount path.
- Pass the lifecycle `View` controller to the `ViewContext::open_view` build closure for
  registration, and to every ordinary `ViewDelegate` callback as `&mut View`. Do not add
  separate `ViewDelegate::mount` or `ViewDelegate::unmount` methods; lifecycle hooks belong to
  the `View` controller and receive the current `&mut App`. A `View` callback that already
  receives `&mut Window` should use that temporary borrow directly; do not store `Window` in
  the `View` controller.
- Keep window-independent operations on `App` or an entity context. Only code that
  renders or handles window-specific input should require `&mut Window`.
- GPUI callbacks and entities are main-thread values unless an API explicitly says
  otherwise. Use `Context::spawn` or `App::background_spawn` for asynchronous work and
  send the result back through a weak entity handle before notifying the UI.
- Do not type-erase a GPUI view by inventing a parallel render protocol. Use
  `AnyView`, `AnyEntity`, or a small adapter that implements `Render`; document the
  adapter when a feature-specific delegate must remain separate from GPUI's `Render`
  trait.
- Use GPUI's `Action` trait for view actions and keybindings. If a heterogeneous action
  list needs type erasure, keep the box inside the view layer and expose a constructor
  that accepts concrete action values.

## Documentation and API Status

Most crates are currently scaffolds. README API blocks describe the intended contract,
not necessarily an already exported implementation. A proposal must say when it relies
on an adapter or on a dependency that is not present yet. Keep examples aligned with
the gpui revision pinned in `Cargo.lock`.

## Technical Perfectionism Rule

When unreasonable design, redundant abstractions, duplicated logic, dead pathways, or
unnecessary code is discovered in the scope being touched or reviewed, do not preserve
it for convenience. Treat root-cause cleanup as required work: remove the redundancy,
simplify the design, and refactor boldly enough to leave the code technically clean
instead of applying a narrow patch over a known flaw.

## Development Workflow

1. Identify the requested capability.
2. Read [ARCHITECTURE.md](./ARCHITECTURE.md) and confirm which crate owns it.
3. Confirm that the owning crate already has the right abstraction; add a new crate only
   when no existing crate fits.
4. Update code in the smallest appropriate package, and update documentation when
   structure, public API, ownership, or rules change.
5. Run `cargo fmt --all -- --check`, `cargo check --workspace`, and any focused tests
   before reporting completion.
