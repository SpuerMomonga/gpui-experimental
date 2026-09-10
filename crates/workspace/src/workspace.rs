use std::rc::Rc;
use std::{collections::HashMap, sync::Arc};

use gpui_kit::component::{
    Root,
    dock::{DockArea, DockSkin},
};
use gpui_kit::{
    App, AppContext, Bounds, Context, Entity, Global, IntoElement, Render, Task, WeakEntity,
    Window, WindowBounds, WindowKind, WindowOptions, div, prelude::*, px, rgb, size,
};

use uuid::Uuid;

mod actions_bar;
mod item;
mod title_bar;

pub use actions_bar::*;
pub use item::*;
pub use title_bar::*;

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
    workspaces: HashMap<WorkspaceId, (gpui_kit::AnyWindowHandle, WeakEntity<Workspace>)>,
}

impl WorkspaceStore {
    pub fn new(cx: &mut App) -> Self {
        Self {
            workspaces: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorkspaceId(i64);

pub struct Workspace {
    title_bar: Entity<TitleBar>,
    center: Entity<DockArea>,
    skin: Rc<DockSkin>,
    actions_bar: Entity<ActionsBar>,
}

impl Workspace {
    pub fn new(app_state: Arc<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (dock_area, skin) = DockSkin::dock_area("workspace-dock", Some(1), window, cx);

        let title_bar = cx.new(|_cx| TitleBar::new());
        let actions_bar = cx.new(|_cx| ActionsBar::new());

        let weak_handle = cx.entity().downgrade();
        let any_window_handle = window.window_handle();
        app_state.workspace_store.update(cx, |store, _| {
            let id = WorkspaceId(Uuid::new_v4().as_u64_pair().0 as i64);
            store
                .workspaces
                .insert(id, (any_window_handle, weak_handle.clone()));
        });

        Self {
            title_bar,
            center: dock_area,
            skin,
            actions_bar,
        }
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .gap_0()
            .overflow_hidden()
            .child(self.title_bar.clone())
            .child(div().flex_1().min_h_0().child(self.center.clone()))
            .child(self.actions_bar.clone())
    }
}

pub fn with_active_or_primary_workspace() {}