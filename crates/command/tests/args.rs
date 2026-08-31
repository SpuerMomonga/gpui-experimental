use command::{Args, FieldType, Input, Kind};
use serde::Deserialize;

#[derive(Debug, Deserialize, Args)]
struct HistoryArgs {
    /// An item to paste.
    #[arg(name = "item", description = "The item id")]
    item_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Args)]
struct OptionalArgs {
    #[arg(default = 3, kind = "integer")]
    count: i32,
    label: Option<String>,
}

#[test]
fn derives_schema_and_decodes_external_input() {
    let schema = HistoryArgs::schema();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields[0].name, "item");
    assert_eq!(schema.fields[0].description.as_deref(), Some("The item id"));
    assert_eq!(schema.fields[0].kind, Kind::String);
    assert!(schema.fields[0].required);

    let args = HistoryArgs::decode(Input::External(serde_json::json!({
        "item_id": "clipboard-1"
    })))
    .unwrap();
    assert_eq!(args.item_id, "clipboard-1");
    assert!(HistoryArgs::decode(Input::Internal(Default::default())).is_err());
    assert!(
        HistoryArgs::decode(Input::External(serde_json::json!({
            "item_id": 42
        })))
        .is_err()
    );
    assert!(HistoryArgs::decode(Input::External(serde_json::json!({}))).is_err());
}

#[test]
fn internal_args_only_accept_internal_input() {
    let args = command::HostArgs::decode(Input::Internal(command::HostArgs {
        query: Some("search".into()),
    }))
    .unwrap();
    assert_eq!(args.query.as_deref(), Some("search"));
    assert!(command::HostArgs::decode(Input::External(serde_json::json!({}))).is_err());
}

#[test]
fn field_type_maps_scalar_types() {
    assert_eq!(<String as FieldType>::kind(), Kind::String);
    assert_eq!(<i32 as FieldType>::kind(), Kind::Integer);
    assert_eq!(<f64 as FieldType>::kind(), Kind::Number);
    assert_eq!(<bool as FieldType>::kind(), Kind::Boolean);
}

#[test]
fn derives_optional_and_default_fields() {
    let fields = OptionalArgs::schema().fields;
    assert!(!fields[0].required);
    assert_eq!(fields[0].kind, Kind::Integer);
    assert_eq!(fields[0].default, Some(serde_json::json!(3)));
    assert!(!fields[1].required);

    let args = OptionalArgs::decode(Input::External(serde_json::json!({}))).unwrap();
    assert_eq!(args.count, 3);
    assert_eq!(args.label, None);

    let args = OptionalArgs::decode(Input::External(serde_json::json!({
        "count": 9,
        "label": "recent"
    })))
    .unwrap();
    assert_eq!(args.count, 9);
    assert_eq!(args.label.as_deref(), Some("recent"));
}

#[allow(dead_code)]
fn registration_type_inference_is_supported(
    registry: &mut command::CommandRegistry,
    cx: &mut gpui::Context<command::CommandRegistry>,
) {
    registry.register(
        command::Command::new("test.command", "Test"),
        |_args: command::HostArgs, _app: &mut gpui::App| Ok(()),
        cx,
    );
}
