use anyhow::{Result, bail};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

/// Metadata shown to users of a command.
///
/// Argument information intentionally lives in [`Schema`] rather than on this
/// type. This keeps command metadata non-generic and lets the registry infer
/// argument handling from the callback passed to [`CommandRegistry::register`].
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

fn default_enabled() -> bool {
    true
}

impl Command {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            enabled: true,
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

/// Runtime description of a command's externally supplied arguments.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    #[serde(default)]
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub kind: Kind,
    pub required: bool,
    #[serde(default)]
    pub default: Option<Value>,
}

impl Field {
    pub fn new(name: impl Into<String>, kind: Kind, required: bool) -> Self {
        Self {
            name: name.into(),
            description: None,
            kind,
            required,
            default: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Kind {
    String,
    Integer,
    Number,
    Boolean,
    Json,
    Enum { values: Vec<EnumValue> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumValue {
    pub value: String,
    pub description: Option<String>,
}

/// Converts a Rust argument type into a JSON-facing field kind.
pub trait FieldType: DeserializeOwned + 'static {
    fn kind() -> Kind;
}

impl FieldType for String {
    fn kind() -> Kind {
        Kind::String
    }
}

impl FieldType for bool {
    fn kind() -> Kind {
        Kind::Boolean
    }
}

macro_rules! integer_field_types {
    ($($type:ty),* $(,)?) => {
        $(
            impl FieldType for $type {
                fn kind() -> Kind {
                    Kind::Integer
                }
            }
        )*
    };
}

integer_field_types!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

macro_rules! number_field_types {
    ($($type:ty),* $(,)?) => {
        $(
            impl FieldType for $type {
                fn kind() -> Kind {
                    Kind::Number
                }
            }
        )*
    };
}

number_field_types!(f32, f64);

impl FieldType for Value {
    fn kind() -> Kind {
        Kind::Json
    }
}

impl<T> FieldType for Option<T>
where
    T: FieldType,
{
    fn kind() -> Kind {
        T::kind()
    }
}

/// A command's decoded argument value.
pub trait Args: Sized + 'static {
    fn schema() -> Schema;
    fn decode(input: Input) -> Result<Self>;
}

/// Host-owned arguments used by commands without externally supplied fields.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostArgs {
    /// The command-palette query, when execution originated from the palette.
    pub query: Option<String>,
}

impl Args for HostArgs {
    fn schema() -> Schema {
        Schema::default()
    }

    fn decode(input: Input) -> Result<Self> {
        match input {
            Input::Internal(args) => Ok(args),
            Input::External(_) => bail!("parameterless command requires internal input"),
        }
    }
}

/// The only input boundary accepted by the command registry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Input {
    External(Value),
    Internal(HostArgs),
}

/// Metadata and generated argument schema for one registered command.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Descriptor {
    pub command: Command,
    pub schema: Schema,
    /// True exactly when the command has no externally supplied fields.
    pub palette_visible: bool,
}

/// A lifecycle change emitted by [`CommandRegistry`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandEvent {
    Registered { id: String },
    Changed { id: String },
    Removed { id: String },
}
