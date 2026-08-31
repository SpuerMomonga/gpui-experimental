# command-palette

## Responsibilities

`command-palette` is the application's default search surface. It consumes the registries
provided by `command`, `preview`, and (when needed) `view`, but it does not define command
handlers or own view lifecycles. This crate remains an empty scaffold; this document
describes its integration boundary rather than an implemented component.

The current `Cargo.toml` has no dependencies. Add `gpui`, `command`, `preview`, and any
required `view` or collection dependencies when implementing the component, and update
`ARCHITECTURE.md` at the same time.

The palette can serve as the top region of `workspace`, but it should remain independent
enough to embed in other GPUI windows. Query and selection state live in the palette entity;
command and preview data remain in their respective registries.

## Query Flow

1. Read enabled, effectively visible commands with an empty parameter schema from
   `CommandRegistry`. The palette can pass only a command ID and cannot display or collect
   custom arguments, so the registry excludes parameterized commands from palette results.
2. Compute a deterministic match score from title, subtitle, category, and keywords. Identify
   the selected item by command ID, never by its list index.
3. If there is no direct result, ask `PreviewRegistry` for its best provider. A preview is a
   fallback result, not another set of command results.
4. When confirming a parameterless command, construct only
   `Input::Internal(HostArgs { query: Some(query) })`. CLI, IPC, or application code should
   invoke parameterized commands with `Input::External(Value)`. Commands and preview
   providers execute on the GPUI application thread. Unless a command explicitly requires a
   different policy, close or replace the host view only after the operation succeeds.

The palette observes registry `CommandEvent` values through `subscribe`. Call its own
`cx.notify()` when the query or selection changes; do not rebuild every provider merely
because the input changed.

## GPUI Boundary

The visual root entity implements `gpui::Render`. Render list rows with ordinary GPUI
elements, and use concretely typed `Action` values for keyboard navigation. Shortcuts in
command metadata are display-only; they do not replace GPUI's keymap and context-priority
mechanisms.
