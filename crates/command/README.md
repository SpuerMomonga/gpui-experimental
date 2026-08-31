# command

## Overview

`command` provides the application-level command registry. Commands do not depend on a
particular window or view, so hotkeys, IPC, and other application-level events can invoke
them before the application has created a window. `command-palette` only searches for and
displays commands without custom arguments; this crate owns command metadata, parameter
schemas, registration lifecycle, and execution logic.

The argument derive macro lives in the `command` module of the shared `macros` proc-macro
crate and is re-exported by `command`. Implementations must follow the GPUI version pinned
in `Cargo.lock`; tests should continue to cover registration, duplicate IDs, argument
decoding, execution failures, and event subscriptions.

## Design Notes

`Command` stores only command metadata and carries no parameter generic. Parameterless and
parameterized commands use the same type. The first parameter type of the registration
callback determines how arguments are supplied: when the callback accepts a custom parameter
struct, the registry decodes external input; when it accepts `HostArgs`, the registry passes
host-constructed internal arguments. Only `register` is generic, so callers do not need to
make `Command` generic or create a separate invocation object.

The input boundary has exactly two branches: `Input::External(Value)` and
`Input::Internal(HostArgs)`. An external `Value` exists only at the CLI, IPC, or application
protocol boundary and is decoded according to the callback parameter type before the
callback runs. Internal arguments are a flat host struct whose fields directly represent
fixed data such as the palette query; they are not looked up by string keys or wrapped in a
source enum.

Because `command-palette` can pass only a command ID, commands with custom argument fields
cannot appear in the palette. The registry computes effective visibility from the schema
generated for the callback parameters: a non-empty schema excludes a command from palette
results, while CLI, IPC, and application code can still execute it.

Command IDs use the `app_id.feature_id` form and must be globally unique within the
application or plugin. Registration returns neither a token nor an error: the first
registration for an ID wins and later duplicates are ignored. Remove a command with the
void-returning `unregister` method. Handlers receive only `&mut App`, not `&mut Window`;
window behavior is performed by calling `view` or `App::open_window` from the handler.

## Parameter Types and Schemas

Parameters are ordinary Rust structs. `#[derive(Args)]` generates the `Args` implementation,
deserialization constraints, and schema. Describe fields with `#[arg(...)]` attributes or
field documentation comments:

```rust
use command::Args;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Args)]
pub struct HistoryArgs {
    /// ID of the clipboard item to paste.
    #[arg(name = "item", description = "ID of the clipboard item to paste")]
    pub item_id: String,
}
```

`Args` is the minimum bound required for registration. The derive implementation generates a
schema from field types and provides input decoding; the schema is used only for registry,
CLI, and IPC help and validation, while handlers receive an already decoded `HistoryArgs`.
Attributes override `name`, `description`, `default`, and `kind`. `Option<T>` and fields with
defaults can be inferred as optional. `command` maps common field types; custom field types
must implement `FieldType` so the macro can generate the corresponding `Kind`.

```rust
pub trait Args: Sized + 'static {
    fn schema() -> Schema;
    fn decode(input: Input) -> anyhow::Result<Self>;
}

pub trait FieldType: serde::de::DeserializeOwned + 'static {
    fn kind() -> Kind;
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Schema {
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub kind: Kind,
    pub required: bool,
    pub default: Option<serde_json::Value>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub enum Kind {
    String,
    Integer,
    Number,
    Boolean,
    Json,
    Enum { values: Vec<EnumValue> },
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct EnumValue {
    pub value: String,
    pub description: Option<String>,
}
```

## Input Types

The registry accepts exactly two input branches:

```rust
use serde_json::Value;

#[derive(Clone, Debug, Default)]
pub struct HostArgs {
    /// Query text when the command palette confirms a command; usually `None` for other callers.
    pub query: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Input {
    /// JSON arguments supplied by CLI, IPC, or application code.
    External(Value),
    /// Fixed arguments constructed directly by the application host.
    Internal(HostArgs),
}

impl Args for HostArgs {
    fn schema() -> Schema {
        Schema { fields: Vec::new() }
    }

    fn decode(input: Input) -> anyhow::Result<Self> {
        match input {
            Input::Internal(args) => Ok(args),
            Input::External(_) => anyhow::bail!("parameterless command requires internal input"),
        }
    }
}
```

`HostArgs` also implements `Args`, but its `schema` is empty and `decode` accepts only
`Input::Internal`. Derived external parameter types likewise implement `Args` but accept
only `Input::External`. The registry therefore keeps input errors out of handlers, which
receive exactly the parameter object they declare.

## Command and Registration API

`Command` contains metadata only; `register`'s generic bounds provide the schema and argument
decoding:

```rust
#[derive(Clone, Debug)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub shortcut: Option<String>,
    pub enabled: bool,
}

pub struct Descriptor {
    pub command: Command,
    pub schema: Schema,
    pub palette_visible: bool,
}

pub struct CommandRegistry {
    // id -> (descriptor, internally type-erased handler)
}

impl CommandRegistry {
    pub fn register<A, F>(
        &mut self,
        command: Command,
        handler: F,
        cx: &mut gpui::Context<Self>,
    ) where
        A: Args,
        F: Fn(A, &mut gpui::App) -> anyhow::Result<()> + 'static;

    pub fn unregister(&mut self, id: &str, cx: &mut gpui::Context<Self>);

    pub fn execute(
        &mut self,
        id: &str,
        input: Input,
        cx: &mut gpui::Context<Self>,
    ) -> anyhow::Result<()>;

    pub fn iter(&self, visible: Option<bool>) -> impl Iterator<Item = &Descriptor>;
}
```

`A` in `register<A, F>` is inferred from the callback's first parameter type; its type bounds
perform both schema discovery and input decoding. The registry may erase callbacks into a
uniform internal call structure, but `Box` and handler type aliases must not appear in the
caller-facing API. `execute` passes input to `Args::decode` and then invokes the callback;
input-branch mismatches return an error and never pass the wrong argument type to a handler.

The following registrations show how the parameter type selects the input source:

```rust
// Custom parameters use Input::External(Value).
registry.register(
    Command::new("clipboard.history", "Clipboard History"),
    |args: HistoryArgs, app: &mut gpui::App| {
        let _item_id = args.item_id;
        let _ = app;
        Ok(())
    },
    cx,
);

// Commands without custom parameters use Input::Internal(HostArgs).
registry.register(
    Command::new("search.focus", "Focus Search"),
    |args: HostArgs, app: &mut gpui::App| {
        let _query = args.query;
        let _ = app;
        Ok(())
    },
    cx,
);
```

During registration, the registry uses `schema.fields.is_empty()` for effective palette
visibility: commands without external parameters are visible, while parameterized commands
are hidden. `enabled` controls whether a command enters the enabled-command list; `execute`
decodes input and invokes the handler. Registration, updates, and removals emit a
`CommandEvent` through the registry's `EventEmitter` implementation. Because the registry
has no visual root entity, these lifecycle operations do not also call `cx.notify()`;
consumers that need to refresh their UI call `cx.notify()` in their own entity context.

## GPUI Boundary

`CommandRegistry` is stored in a GPUI `Entity<CommandRegistry>` and exposed through a private
`Global` newtype. The `cx` parameter of `register` is a `Context<CommandRegistry>`; handlers
receive `&mut App` only when they execute. Shortcut metadata is display-only; real shortcuts
use GPUI's `Action`, `KeyBinding`, and `App::on_action`. Initialize the command registry
before calling `global`; the application entry point owns initialization order.

To keep application code from repeatedly retrieving the global entity, `command` also
provides the `CommandContext` extension trait. After importing it, call `register`,
`unregister`, `execute`, and `update` directly on `&mut gpui::App`; all four methods forward
to the initialized global `CommandRegistry`.
