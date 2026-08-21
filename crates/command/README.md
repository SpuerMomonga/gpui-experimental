## Overview

The command system is a global, application-level registry. All commands are registered once through gpui and are guaranteed to be globally unique, with each command bound to a single handler.

Commands are decoupled from any specific window or view. They can also be invoked externally via a cli + ipc channel that communicates with the main thread and dispatches commands there.

By default, the application starts without opening any window. Windows are only created when the user wakes the app with a hotkey. Commands must remain fully callable even when no window exists.

> **Command ID Convention:** `command.id` follows the format `app_id.feature_id`. The `app_id` must be unique across the application. If a `command.id` is duplicated, only the first registration takes effect; subsequent registrations with the same ID are ignored.

## Data Model

```rust

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub icon: Option<Icon>,
    pub keywords: Option<Vec<String>>,
    pub shortcut: Option<String>,
    pub disabled: bool,
    pub visible: bool,
}

```

## API
 
```rust

pub trait CommandAction: Fn(&Params, &mut App) + 'static {}

impl<F> CommandAction for F where F: Fn(&Params, &mut App) + 'static {}

```

The CommandRegistry is exposed through an extension trait on App for ergonomic access.

```rust

pub trait CommandContext {
    fn register_command(&mut self, command: Command, action: CommandAction);
    fn unregister_command(&mut self, command_id: &str);
}

impl CommandContext for App {
    fn register_command(&mut self, command: Command, action: CommandAction) {
        todo!()
    }

    fn unregister_command(&mut self, command_id: &str) {
        todo!()
    }
}

```

`CommandRegistry`

```rust

pub struct CommandRegistry {
    // 
}

impl CommandRegistry {
    pub fn register_command(&mut self, command: Command, action: CommandAction, cx: &mut App) {
        todo!()
    }

    pub fn unregister_command(&mut self, command_id: &str, cx: &mut App) {
        todo!()
    }

    pub fn execute_command(&mut self, command_id: &str, params: &Params, cx: &mut App) -> anyhow::Result<()> {
        todo!()
    }

    pub fn iter_command(&mut self, visible: Option<bool>) -> impl Iterator<Item = &Command> {
        todo!()
    }

    pub fn register_action(&mut self, command_id: &str, action: CommandAction) {
        todo!()
    }
}

```

## Examples

Using the registry directly

```rust

pub fn init(cx: &mut App) {
    Clipboard::register(cx);
}

pub struct Clipboard;

impl Clipboard {
    fn register(cx: &mut App) {
        CommandRegistry::global(cx).update(cx, |registry, cx| {
            let command = Command::new("clipboard.history", "Clipboard History")
                .description("Open clipboard history and select an item to paste.")
                .keywords(["copy", "history"]);

            registry.register_command(command, |params, cx| {}, cx);
        })
    }
}

```

Using the extension trait

```rust

pub fn init(cx: &mut App) {
    Clipboard::register(cx);
}

pub struct Clipboard;

impl Clipboard {
    fn register(cx: &mut App) {
        let command = Command::new("clipboard.history", "Clipboard History")
            .description("Open clipboard history and select an item to paste.")
            .keywords(["copy", "history"]);

        cx.register_command(command, |params, cx| {});
    }
}

```