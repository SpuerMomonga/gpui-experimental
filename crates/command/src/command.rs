use gpui_kit::{App, AppContext as _, Entity, EventEmitter, Global};
use serde::{Deserialize, Serialize};

pub fn init(cx: &mut App) {
    let registry = cx.new(|_| CommandRegistry::new());
    cx.set_global(GlobalCommandRegistry(registry));
}

struct GlobalCommandRegistry(Entity<CommandRegistry>);

impl Global for GlobalCommandRegistry {}

pub struct CommandRegistry {}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {}
    }

    pub fn global(cx: &App) -> Entity<CommandRegistry> {
        cx.global::<GlobalCommandRegistry>().0.clone()
    }

    pub fn register(command: Command, cx: &mut App) {
        CommandRegistry::global(cx).update(cx, |registry, cx| {
            if registry.insert(command) {
                cx.emit(CommandEvent::Added);
            }
        });
    }
}

impl CommandRegistry {
    fn insert(&mut self, command: Command) -> bool {
        todo!()
    }
}

pub enum CommandEvent {
    Added,
    Changed,
    Deleted,
}

impl EventEmitter<CommandEvent> for CommandRegistry {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub shortcut: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl Default for Command {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            subtitle: None,
            category: None,
            description: None,
            keywords: Vec::new(),
            shortcut: None,
            enabled: true,
        }
    }
}

impl Command {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            ..Default::default()
        }
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
