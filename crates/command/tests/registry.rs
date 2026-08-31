use command::{Args, Command, CommandContext, CommandEvent, CommandRegistry, HostArgs, Input};
use gpui::{AppContext, TestAppContext};
use serde::Deserialize;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Deserialize, Args)]
struct HistoryArgs {
    item_id: String,
}

#[test]
fn registration_preserves_first_duplicate_and_order() {
    let mut cx = TestAppContext::single();
    let registry = cx.new(|_cx| CommandRegistry::new());

    registry.update(&mut cx, |registry, cx| {
        registry.register(
            Command::new("app.first", "First"),
            |_args: HostArgs, _app| Ok(()),
            cx,
        );
        registry.register(
            Command::new("app.second", "Second"),
            |_args: HostArgs, _app| Ok(()),
            cx,
        );
        registry.register(
            Command::new("app.first", "Replacement"),
            |_args: HostArgs, _app| Ok(()),
            cx,
        );
    });

    registry.read_with(&cx, |registry, _app| {
        let ids = registry
            .iter(None)
            .map(|descriptor| descriptor.command.id.as_str());
        assert_eq!(ids.collect::<Vec<_>>(), ["app.first", "app.second"]);
        assert_eq!(registry.get("app.first").unwrap().command.title, "First");
    });
}

#[test]
fn unregister_allows_reregistration_and_moves_command_to_the_end() {
    let mut cx = TestAppContext::single();
    let registry = cx.new(|_cx| CommandRegistry::new());

    registry.update(&mut cx, |registry, cx| {
        registry.register(
            Command::new("app.first", "First"),
            |_args: HostArgs, _app| Ok(()),
            cx,
        );
        registry.register(
            Command::new("app.second", "Second"),
            |_args: HostArgs, _app| Ok(()),
            cx,
        );
        registry.unregister("app.first", cx);
        registry.register(
            Command::new("app.first", "Replacement"),
            |_args: HostArgs, _app| Ok(()),
            cx,
        );
    });

    registry.read_with(&cx, |registry, _app| {
        let ids = registry
            .iter(None)
            .map(|descriptor| descriptor.command.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, ["app.second", "app.first"]);
        assert_eq!(
            registry.get("app.first").unwrap().command.title,
            "Replacement"
        );
    });
}

#[test]
fn schema_controls_palette_visibility_and_enabled_filters_results() {
    let mut cx = TestAppContext::single();
    let registry = cx.new(|_cx| CommandRegistry::new());

    registry.update(&mut cx, |registry, cx| {
        registry.register(
            Command::new("app.visible", "Visible"),
            |_args: HostArgs, _app| Ok(()),
            cx,
        );
        registry.register(
            Command::new("app.with-args", "With args"),
            |_args: HistoryArgs, _app| Ok(()),
            cx,
        );
        registry.register(
            Command::new("app.disabled", "Disabled").with_enabled(false),
            |_args: HostArgs, _app| Ok(()),
            cx,
        );
    });

    registry.read_with(&cx, |registry, _app| {
        assert!(registry.get("app.visible").unwrap().palette_visible);
        assert!(!registry.get("app.with-args").unwrap().palette_visible);
        assert!(!registry.get("app.disabled").unwrap().command.enabled);
        let enabled_ids = registry
            .iter(Some(true))
            .map(|descriptor| descriptor.command.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(enabled_ids, ["app.visible"]);
        assert_eq!(registry.iter(Some(false)).count(), 2);
    });
}

#[test]
fn execute_decodes_the_registered_input_and_propagates_errors() {
    let mut cx = TestAppContext::single();
    let registry = cx.new(|_cx| CommandRegistry::new());
    let calls = Rc::new(RefCell::new(Vec::<String>::new()));
    let observed_calls = calls.clone();

    registry.update(&mut cx, |registry, cx| {
        registry.register(
            Command::new("app.history", "History"),
            move |args: HistoryArgs, _app| {
                observed_calls.borrow_mut().push(args.item_id);
                Ok(())
            },
            cx,
        );
    });

    registry.update(&mut cx, |registry, cx| {
        registry
            .execute(
                "app.history",
                Input::External(serde_json::json!({ "item_id": "one" })),
                cx,
            )
            .unwrap();
        assert!(
            registry
                .execute("app.history", Input::Internal(HostArgs::default()), cx)
                .is_err()
        );
        assert!(
            registry
                .execute(
                    "app.history",
                    Input::External(serde_json::json!({ "item_id": 7 })),
                    cx,
                )
                .is_err()
        );
        assert!(
            registry
                .execute("app.unknown", Input::External(serde_json::json!({})), cx,)
                .is_err()
        );
    });

    assert_eq!(&*calls.borrow(), &["one"]);
}

#[test]
fn command_events_cover_registration_changes_and_removal() {
    let mut cx = TestAppContext::single();
    let registry = cx.new(|_cx| CommandRegistry::new());
    let events = Rc::new(RefCell::new(Vec::<CommandEvent>::new()));
    let observed_events = events.clone();
    let _subscription = cx.update(|app| {
        app.subscribe(&registry, move |_entity, event, _app| {
            observed_events.borrow_mut().push(event.clone());
        })
    });
    cx.run_until_parked();

    registry.update(&mut cx, |registry, cx| {
        registry.register(
            Command::new("app.command", "Command"),
            |_args: HostArgs, _app| Ok(()),
            cx,
        );
    });
    registry.update(&mut cx, |registry, cx| {
        registry
            .update(
                "app.command",
                |command| command.title = "Updated".to_owned(),
                cx,
            )
            .unwrap();
    });
    registry.update(&mut cx, |registry, cx| {
        registry.unregister("app.command", cx);
        registry.unregister("app.command", cx);
    });
    cx.run_until_parked();

    assert_eq!(
        &*events.borrow(),
        &[
            CommandEvent::Registered {
                id: "app.command".to_owned()
            },
            CommandEvent::Changed {
                id: "app.command".to_owned()
            },
            CommandEvent::Removed {
                id: "app.command".to_owned()
            },
        ]
    );
}

#[test]
fn metadata_updates_reject_id_changes_and_missing_commands() {
    let mut cx = TestAppContext::single();
    let registry = cx.new(|_cx| CommandRegistry::new());

    registry.update(&mut cx, |registry, cx| {
        registry.register(
            Command::new("app.command", "Command"),
            |_args: HostArgs, _app| Ok(()),
            cx,
        );
        assert!(
            registry
                .update("app.command", |command| command.id = "other".to_owned(), cx)
                .is_err()
        );
        assert!(registry.update("app.missing", |_| {}, cx).is_err());
    });

    registry.read_with(&cx, |registry, _app| {
        assert_eq!(
            registry.get("app.command").unwrap().command.id,
            "app.command"
        );
    });
}

#[test]
fn init_installs_a_global_registry() {
    let cx = TestAppContext::single();
    assert!(cx.read(|app| CommandRegistry::try_global(app)).is_none());

    cx.update(|app| command::init(app));

    let registry = cx
        .read(|app| CommandRegistry::try_global(app))
        .expect("command::init should install a global registry");
    registry.read_with(&cx, |registry, _app| {
        assert_eq!(registry.iter(None).count(), 0);
    });
}

#[test]
fn app_extension_forwards_all_registry_operations() {
    let cx = TestAppContext::single();
    let calls = Rc::new(RefCell::new(Vec::<String>::new()));
    let observed_calls = calls.clone();

    cx.update(command::init);
    cx.update(|app| {
        app.register(
            Command::new("app.history", "History"),
            move |args: HistoryArgs, _app| {
                observed_calls.borrow_mut().push(args.item_id);
                Ok(())
            },
        );
        app.update("app.history", |command| {
            command.title = "Updated".to_owned()
        })
        .unwrap();
        app.execute(
            "app.history",
            Input::External(serde_json::json!({ "item_id": "from-app" })),
        )
        .unwrap();

        let registry = CommandRegistry::global(app);
        assert_eq!(
            registry.read(app).get("app.history").unwrap().command.title,
            "Updated"
        );
        app.unregister("app.history");
        assert!(registry.read(app).get("app.history").is_none());
    });

    assert_eq!(&*calls.borrow(), &["from-app"]);
}
