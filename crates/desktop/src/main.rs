use std::sync::Arc;

use gpui_kit::component::theme::{Theme, ThemeMode};
use gpui_kit::{AppContext as _, Application, platform};

use assets::Assets;
use workspace::{AppState, Workspace, WorkspaceStore};

fn main() {
    Application::with_platform(platform::current_platform(false))
        .with_assets(Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            Theme::change(ThemeMode::Light, None, cx);

            command::init(cx);

            let workspace_store = cx.new(|cx| WorkspaceStore::new(cx));

            let app_state = Arc::new(AppState { workspace_store });
            AppState::set_global(app_state.clone(), cx);

            Workspace::new_local(app_state, cx).detach();

            command_palette::init(cx);
        });
}
