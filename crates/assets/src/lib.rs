use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../assets"]
#[include = "fonts/**/*"]
#[include = "icons/**/*"]
#[include = "images/**/*"]
#[include = "themes/**/*"]
#[exclude = "*.DS_Store"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        if let Some(file) = Self::get(path) {
            return Ok(Some(file.data));
        }

        gpui_component_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut result: Vec<SharedString> = Self::iter()
            .filter_map(|p| {
               p.starts_with(path)
                    .then(|| SharedString::from(p.to_string()))
            })
            .collect();
        
        if let Ok(component_assets) = gpui_component_assets::Assets.list(path) {
            result.extend(component_assets);
        }

        Ok(result)
    }
}