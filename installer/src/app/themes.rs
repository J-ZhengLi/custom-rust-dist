use iced::widget::container;
use iced::{Color, Theme};

const PANEL_BG: Color = Color::from_rgb(0.94, 0.94, 0.94);
const PANEL_BG_DARK: Color = Color::from_rgb(0.22, 0.22, 0.22);

pub(crate) struct PanelTheme;

impl container::StyleSheet for PanelTheme {
    type Style = Theme;
    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Theme::Light => container::Appearance {
                background: Some(PANEL_BG.into()),
                ..Default::default()
            },
            Theme::Dark => container::Appearance {
                background: Some(PANEL_BG_DARK.into()),
                ..Default::default()
            },
            _ => unimplemented!("custom theme is not supported yet"),
        }
    }
}
