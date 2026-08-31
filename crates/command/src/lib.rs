use collections::HashMap;
use gpui::{App, AppContext, Context, Entity, EventEmitter, Global};

// Re-exported for the generated Args implementation emitted by the proc macro.
#[doc(hidden)]
pub extern crate anyhow;
extern crate self as command;
#[doc(hidden)]
pub extern crate serde_json;

mod types;

pub use macros::Args;
pub use types::*;

use anyhow::{Result, anyhow, bail};

type Handler = Box<dyn Fn(Input, &mut App) -> Result<()> + 'static>;

/// Main-thread command registry. Registration order is retained for iteration.
pub struct CommandRegistry {
    commands: HashMap<String, Descriptor>,
    handlers: HashMap<String, Handler>,
    order: Vec<String>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::default(),
            handlers: HashMap::default(),
            order: Vec::new(),
        }
    }

    /// Registers the first command for an ID. Later duplicate registrations are ignored.
    pub fn register<A, F>(&mut self, command: Command, handler: F, cx: &mut Context<Self>)
    where
        A: Args,
        F: Fn(A, &mut App) -> Result<()> + 'static,
    {
        if self.commands.contains_key(&command.id) {
            return;
        }

        let id = command.id.clone();
        let schema = A::schema();
        let palette_visible = schema.fields.is_empty();
        let descriptor = Descriptor {
            command,
            schema,
            palette_visible,
        };
        let erased_handler: Handler = Box::new(move |input, app| {
            let args = A::decode(input)?;
            handler(args, app)
        });

        self.order.push(id.clone());
        self.commands.insert(id.clone(), descriptor);
        self.handlers.insert(id.clone(), erased_handler);
        cx.emit(CommandEvent::Registered { id });
    }

    /// Removes a command and emits an event only when the ID was registered.
    pub fn unregister(&mut self, id: &str, cx: &mut Context<Self>) {
        if self.commands.remove(id).is_none() {
            return;
        }
        self.handlers.remove(id);
        self.order.retain(|registered_id| registered_id != id);
        cx.emit(CommandEvent::Removed { id: id.to_owned() });
    }

    /// Executes a command after decoding the input for its registered argument type.
    pub fn execute(&mut self, id: &str, input: Input, cx: &mut Context<Self>) -> Result<()> {
        let handler = self
            .handlers
            .get(id)
            .ok_or_else(|| anyhow!("command `{id}` is not registered"))?;
        handler(input, &mut **cx)
    }

    /// Updates metadata while preserving the registered handler and argument schema.
    pub fn update(
        &mut self,
        id: &str,
        update: impl FnOnce(&mut Command),
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let entry = self
            .commands
            .get_mut(id)
            .ok_or_else(|| anyhow!("command `{id}` is not registered"))?;
        let before = entry.command.clone();
        update(&mut entry.command);
        if entry.command.id != id {
            entry.command = before;
            bail!("command ID cannot be changed");
        }
        if entry.command != before {
            cx.emit(CommandEvent::Changed { id: id.to_owned() });
        }
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Descriptor> {
        self.commands.get(id)
    }

    /// Iterates in registration order. `Some(true)` filters palette-visible and
    /// enabled commands; `Some(false)` returns the rest; `None` returns all.
    pub fn iter(&self, visible: Option<bool>) -> impl Iterator<Item = &Descriptor> {
        self.order
            .iter()
            .filter_map(|id| self.commands.get(id))
            .filter(move |descriptor| match visible {
                Some(true) => descriptor.palette_visible && descriptor.command.enabled,
                Some(false) => !descriptor.palette_visible || !descriptor.command.enabled,
                None => true,
            })
    }
}

impl EventEmitter<CommandEvent> for CommandRegistry {}

/// Ergonomic command operations backed by the application's global registry.
pub trait CommandContext {
    fn register<A, F>(&mut self, command: Command, handler: F)
    where
        A: Args,
        F: Fn(A, &mut App) -> Result<()> + 'static;

    fn unregister(&mut self, id: &str);

    fn execute(&mut self, id: &str, input: Input) -> Result<()>;

    fn update(&mut self, id: &str, update: impl FnOnce(&mut Command)) -> Result<()>;
}

impl CommandContext for App {
    fn register<A, F>(&mut self, command: Command, handler: F)
    where
        A: Args,
        F: Fn(A, &mut App) -> Result<()> + 'static,
    {
        let registry = CommandRegistry::global(self);
        registry.update(self, |registry, cx| registry.register(command, handler, cx));
    }

    fn unregister(&mut self, id: &str) {
        let registry = CommandRegistry::global(self);
        registry.update(self, |registry, cx| registry.unregister(id, cx));
    }

    fn execute(&mut self, id: &str, input: Input) -> Result<()> {
        let registry = CommandRegistry::global(self);
        registry.update(self, |registry, cx| registry.execute(id, input, cx))
    }

    fn update(&mut self, id: &str, update: impl FnOnce(&mut Command)) -> Result<()> {
        let registry = CommandRegistry::global(self);
        registry.update(self, |registry, cx| registry.update(id, update, cx))
    }
}

pub fn init(cx: &mut App) {
    let registry = cx.new(|_cx| CommandRegistry::new());
    cx.set_global(GlobalCommandRegistry(registry));
}

impl CommandRegistry {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalCommandRegistry>().0.clone()
    }

    pub fn try_global(cx: &App) -> Option<Entity<Self>> {
        cx.try_global::<GlobalCommandRegistry>()
            .map(|global| global.0.clone())
    }
}

struct GlobalCommandRegistry(Entity<CommandRegistry>);

impl Global for GlobalCommandRegistry {}
