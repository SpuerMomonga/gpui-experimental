use gpui_kit::component::theme::{Theme, ThemeMode};
use gpui_kit::{Application, platform};

use assets::Assets;
use ipc::server::prepare_socket;

fn main() {
    ktracing::init();

    if let Err(_) = prepare_socket() {
        // TODO 通过ipc处理命令并直接返回
        return;
    }

    Application::with_platform(platform::current_platform(false))
        .with_assets(Assets)
        .run(move |cx| {
            i18n::init("en");
            gpui_kit::init(cx);
            Theme::change(ThemeMode::Light, None, cx);

            command::init(cx);

            command_palette::init(cx);
            clipboard::init(cx);
        });
}
