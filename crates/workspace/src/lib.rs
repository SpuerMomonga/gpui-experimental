use std::sync::Arc;

use anyhow::Ok;
use collections::HashSet;
use gpui::{
    App, AppContext, Bounds, Context, Entity, Global, IntoElement, Render, Task, WeakEntity,
    Window, WindowBounds, WindowKind, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_component::{Root, Sizable, input::InputState};

pub fn init(cx: &mut App) {}

pub struct AppState {
    pub workspace_store: Entity<WorkspaceStore>,
}

impl AppState {
    pub fn global(cx: &App) -> Arc<Self> {
        cx.global::<GlobalAppState>().0.clone()
    }

    pub fn try_global(cx: &App) -> Option<Arc<Self>> {
        cx.try_global::<GlobalAppState>()
            .map(|state| state.0.clone())
    }

    pub fn set_global(state: Arc<AppState>, cx: &mut App) {
        cx.set_global(GlobalAppState(state));
    }
}

struct GlobalAppState(Arc<AppState>);

impl Global for GlobalAppState {}

pub struct WorkspaceStore {
    main_workspace: Option<(gpui::AnyWindowHandle, WeakEntity<Workspace>)>,
    workspaces: HashSet<(gpui::AnyWindowHandle, WeakEntity<Workspace>)>,
}

impl WorkspaceStore {
    pub fn new() -> Self {
        Self {
            main_workspace: None,
            workspaces: Default::default(),
        }
    }

    pub fn main_workspace(&self) -> Option<&WeakEntity<Workspace>> {
        self.main_workspace.as_ref().map(|(_, weak)| weak)
    }

    pub fn workspaces(&self) -> impl Iterator<Item = &WeakEntity<Workspace>> {
        self.workspaces.iter().map(|(_, weak)| weak)
    }
}

pub struct Workspace {
    pub(crate) input_state: Entity<InputState>,
}

impl Workspace {
    pub fn new(app_state: Arc<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search for apps and commands..."));

        let weak_handle = cx.entity().downgrade();

        let any_window_handle = window.window_handle();
        app_state.workspace_store.update(cx, |store, _| {
            store
                .workspaces
                .insert((any_window_handle, weak_handle.clone()))
        });

        Self { input_state }
    }

    pub fn new_local(app_state: Arc<AppState>, cx: &mut App) -> Task<anyhow::Result<()>> {
        let display_id = cx.primary_display().map(|display| display.id());
        let bounds =
            WindowBounds::Windowed(Bounds::centered(display_id, size(px(750.), px(475.)), cx));

        cx.spawn(async move |cx| {
            let options = WindowOptions {
                focus: true,
                window_bounds: Some(bounds),
                titlebar: None,
                is_movable: false,
                kind: WindowKind::PopUp,
                display_id,
                ..Default::default()
            };

            cx.open_window(options, {
                let app_state = app_state.clone();
                move |window, cx| {
                    let workspace = cx.new(|cx| Workspace::new(app_state, window, cx));
                    cx.new(|cx| Root::new(workspace, window, cx))
                }
            })?;

            Ok(())
        })
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xFFFFFF))
            .overflow_hidden()
            .child(
                div()
                    .w_full()
                    .p_2()
                    .border_b_1()
                    .border_color(rgb(0xCCCCCC))
                    .child(
                        gpui_component::input::Input::new(&self.input_state)
                            .large()
                            .appearance(false)
                            .cleanable(true),
                    ),
            )
            // view
            .child(div().flex_1().w_full())
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .px_2()
                    .border_t_1()
                    .border_color(rgb(0xCCCCCC))
                    // title
                    .child(div())
                    // actions button and action dropdown
                    .child(div()),
            )
    }
}
