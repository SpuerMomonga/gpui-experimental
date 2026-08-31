use assets::Assets;
use gpui::{AppContext as _, Application};
use gpui_component::theme::{Theme, ThemeMode};
use std::sync::Arc;
use workspace::{AppState, Workspace, WorkspaceStore};

fn main() {
    Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(Assets)
        .run(move |cx| {
            gpui_component::init(cx);
            Theme::change(ThemeMode::Light, None, cx);

            command::init(cx);

            let workspace_store = cx.new(|_cx| WorkspaceStore::new());

            let app_state = Arc::new(AppState { workspace_store });
            AppState::set_global(app_state.clone(), cx);

            Workspace::new_local(app_state, cx).detach();
        });
}
