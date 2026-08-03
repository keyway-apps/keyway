use std::path::{Path, PathBuf};

use gpui::{
    AnyElement, App, Hsla, ImageSource, IntoElement, ParentElement, RenderOnce, SharedString, StyleRefinement, Styled, Window, div, img, prelude::FluentBuilder as _, rgba,
};
use gpui_component::{ActiveTheme, Icon as ComponentIcon, IconName, IconNamed as _, StyledExt};
use serde::{Deserialize, Serialize};

/// A color that can adapt to the active light or dark theme.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct IconColor {
    light: Hsla,
    dark: Hsla,
}

impl IconColor {
    pub const fn new(light: Hsla, dark: Hsla) -> Self {
        Self { light, dark }
    }

    pub const fn light(self) -> Hsla {
        self.light
    }

    pub const fn dark(self) -> Hsla {
        self.dark
    }

    fn resolve(self, is_dark: bool) -> Hsla {
        if is_dark { self.dark } else { self.light }
    }
}

impl From<Hsla> for IconColor {
    fn from(color: Hsla) -> Self {
        Self::new(color, color)
    }
}

impl From<(Hsla, Hsla)> for IconColor {
    fn from((light, dark): (Hsla, Hsla)) -> Self {
        Self::new(light, dark)
    }
}

/// Serializable content supported by an [`Icon`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum IconSource {
    /// A single Unicode scalar value.
    Character(char),
    /// An embedded asset path, URL, or absolute file-system path.
    Path(String),
    /// The asset name of an icon supplied by `gpui_component`.
    #[serde(rename = "builtin")]
    BuiltIn(String),
}

impl From<char> for IconSource {
    fn from(character: char) -> Self {
        Self::Character(character)
    }
}

impl From<&char> for IconSource {
    fn from(character: &char) -> Self {
        Self::Character(*character)
    }
}

impl From<PathBuf> for IconSource {
    fn from(path: PathBuf) -> Self {
        Self::Path(path.to_string_lossy().into_owned())
    }
}

impl From<&Path> for IconSource {
    fn from(path: &Path) -> Self {
        Self::Path(path.to_string_lossy().into_owned())
    }
}

impl From<IconName> for IconSource {
    fn from(icon: IconName) -> Self {
        Self::BuiltIn(builtin_name(icon.path().as_ref()).to_owned())
    }
}

impl From<SharedString> for IconSource {
    fn from(value: SharedString) -> Self {
        source_from_string(value)
    }
}

impl From<&SharedString> for IconSource {
    fn from(value: &SharedString) -> Self {
        source_from_string(value.clone())
    }
}

impl From<String> for IconSource {
    fn from(value: String) -> Self {
        source_from_string(value.into())
    }
}

impl From<&String> for IconSource {
    fn from(value: &String) -> Self {
        source_from_string(value.clone().into())
    }
}

impl From<&str> for IconSource {
    fn from(value: &str) -> Self {
        source_from_string(value.to_owned().into())
    }
}

fn source_from_string(value: SharedString) -> IconSource {
    let mut characters = value.chars();
    if let (Some(character), None) = (characters.next(), characters.next()) {
        return IconSource::Character(character);
    }

    IconSource::Path(value.to_string())
}

fn builtin_name(path: &str) -> &str {
    let name = path.strip_prefix("icons/").unwrap_or(path);
    name.strip_suffix(".svg").unwrap_or(name)
}

fn builtin_path(name: &str) -> SharedString {
    format!("icons/{name}.svg").into()
}

fn image_source(path: String) -> ImageSource {
    if Path::new(&path).is_absolute() {
        PathBuf::from(path).into()
    } else {
        path.into()
    }
}

fn text_content(text: impl Into<SharedString>) -> AnyElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .child(text.into())
        .into_any_element()
}

/// A command-oriented icon that renders text, images, or `gpui_component` icons.
///
/// String values containing exactly one Unicode scalar value render as a character. Other string
/// values are treated as embedded asset paths or URLs, which makes `Command::icon` values directly
/// usable. Absolute paths are loaded from the file system.
#[derive(Clone, Debug, IntoElement, Serialize, Deserialize)]
pub struct Icon {
    source: IconSource,
    #[serde(skip)]
    style: StyleRefinement,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    text_color: Option<IconColor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    background_color: Option<IconColor>,
}

impl Icon {
    pub fn new(source: impl Into<IconSource>) -> Self {
        Self::from_source(source.into())
    }

    pub fn character(character: char) -> Self {
        Self::new(character)
    }

    /// Creates an icon from an embedded path, URL, or absolute file-system path.
    ///
    /// GPUI supports SVG, PNG, JPEG, and its other registered image formats.
    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::from_source(path.into().into())
    }

    pub fn component(icon: IconName) -> Self {
        Self::new(icon)
    }

    pub fn builtin(name: impl Into<String>) -> Self {
        Self::from_source(IconSource::BuiltIn(name.into()))
    }

    pub fn source(&self) -> &IconSource {
        &self.source
    }

    /// Uses one text color in both light and dark themes.
    pub fn text_color(mut self, color: impl Into<IconColor>) -> Self {
        self.text_color = Some(color.into());
        self
    }

    pub fn text_colors(mut self, light: Hsla, dark: Hsla) -> Self {
        self.text_color = Some(IconColor::new(light, dark));
        self
    }

    /// Uses one background color in both light and dark themes.
    pub fn background_color(mut self, color: impl Into<IconColor>) -> Self {
        self.background_color = Some(color.into());
        self
    }

    pub fn background_colors(mut self, light: Hsla, dark: Hsla) -> Self {
        self.background_color = Some(IconColor::new(light, dark));
        self
    }

    fn from_source(source: IconSource) -> Self {
        Self {
            source,
            style: StyleRefinement::default(),
            text_color: None,
            background_color: None,
        }
    }
}

impl Styled for Icon {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let is_dark = cx.theme().is_dark();
        let content = match self.source {
            IconSource::Character(character) => text_content(character.to_string()),
            IconSource::Path(path) => img(image_source(path)).size_full().into_any_element(),
            IconSource::BuiltIn(name) => ComponentIcon::empty()
                .path(builtin_path(&name))
                .into_any_element(),
        };

        div()
            .size_5()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .rounded_sm()
            .bg(rgba(0xDC3432FF))
            .when_some(self.text_color, |this, color| {
                this.text_color(color.resolve(is_dark))
            })
            .when_some(self.background_color, |this, color| {
                this.bg(color.resolve(is_dark))
            })
            .child(content)
            .refine_style(&self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_character_command_icons() {
        assert!(matches!(IconSource::from("K"), IconSource::Character('K')));
        assert!(matches!(
            IconSource::from("\u{952e}"),
            IconSource::Character('\u{952e}')
        ));
    }

    #[test]
    fn treats_non_character_command_icons_as_paths() {
        assert!(matches!(
            IconSource::from("icons/keyway.svg"),
            IconSource::Path(_)
        ));
        assert!(matches!(
            IconSource::from("https://example.com/icon.png"),
            IconSource::Path(_)
        ));
    }

    #[test]
    fn accepts_gpui_component_icons() {
        assert!(matches!(
            IconSource::from(IconName::Search),
            IconSource::BuiltIn(name) if name == "search"
        ));
    }

    #[test]
    fn serializes_path_values_only() {
        let icon = Icon::path("icons/keyway.png");
        let value = serde_json::to_value(&icon).unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "source": { "type": "path", "value": "icons/keyway.png" }
            })
        );

        let restored: Icon = serde_json::from_value(value).unwrap();
        assert_eq!(restored.source, IconSource::Path("icons/keyway.png".into()));
    }

    #[test]
    fn serializes_character_and_theme_colors() {
        let light_text = gpui::rgb(0x111111).into();
        let dark_text = gpui::rgb(0xeeeeee).into();
        let light_background = gpui::rgb(0xffffff).into();
        let dark_background = gpui::rgb(0x222222).into();
        let icon = Icon::character('K')
            .text_colors(light_text, dark_text)
            .background_colors(light_background, dark_background);

        let value = serde_json::to_value(&icon).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "source": { "type": "character", "value": "K" },
                "text_color": { "light": "#111111ff", "dark": "#eeeeeeff" },
                "background_color": { "light": "#ffffffff", "dark": "#222222ff" }
            })
        );

        let restored: Icon = serde_json::from_value(value).unwrap();
        assert_eq!(restored.source, IconSource::Character('K'));
        assert_eq!(restored.text_color, icon.text_color);
        assert_eq!(restored.background_color, icon.background_color);
    }

    #[test]
    fn serializes_builtin_icons_by_name() {
        let icon = Icon::component(IconName::ArrowLeft);
        let value = serde_json::to_value(&icon).unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "source": { "type": "builtin", "value": "arrow-left" }
            })
        );

        let restored: Icon = serde_json::from_value(value).unwrap();
        assert_eq!(restored.source, IconSource::BuiltIn("arrow-left".into()));
    }

    #[test]
    fn resolves_theme_colors() {
        let light = gpui::rgb(0xffffff).into();
        let dark = gpui::rgb(0x111111).into();
        let colors = IconColor::new(light, dark);

        assert_eq!(colors.resolve(false), light);
        assert_eq!(colors.resolve(true), dark);
    }
}
