use super::themes::PanelTheme;
use super::{AppMessage, ThemeMode};
use iced::widget::{self, button, column, container, image, text, toggler, Container};
use iced::{alignment, theme, Alignment, Element, Length, Padding};

#[derive(Debug, Default)]
pub(crate) struct SidePanel;

#[derive(Debug, Clone, Copy)]
enum ButtonStyle {
    Primary,
    Secondary,
}

impl ButtonStyle {
    fn to_style(&self) -> theme::Button {
        match self {
            Self::Primary => theme::Button::Primary,
            Self::Secondary => theme::Button::Secondary,
        }
    }
}

fn panel_button(
    text: &str,
    style: ButtonStyle,
    width: u16,
    msg: AppMessage,
) -> widget::button::Button<'_, AppMessage> {
    button(text)
        .style(style.to_style())
        .width(width)
        .padding(10)
        .on_press(msg)
}

impl<'a> SidePanel {
    pub fn view(&self, darkmode: bool) -> Container<'a, AppMessage> {
        let logo =
            image(format!("{}/res/logo.png", env!("CARGO_MANIFEST_DIR"))).width(Length::Fill);
        let logo_text = text(env!("CARGO_PKG_NAME").to_uppercase())
            .size(21)
            .width(Length::Fill)
            .horizontal_alignment(alignment::Horizontal::Center);
        let brand: Element<_> = column![logo, logo_text]
            .spacing(10)
            .width(Length::Fill)
            .into();

        let toolchains_btn = panel_button(
            "Toolchains",
            ButtonStyle::Primary,
            100,
            AppMessage::ShowToolchains,
        );
        let settings_btn = panel_button(
            "Settings",
            ButtonStyle::Primary,
            100,
            AppMessage::ShowSettings,
        );
        let exit_btn = panel_button("Exit", ButtonStyle::Secondary, 100, AppMessage::Exit);
        let buttons: Element<_> = column![toolchains_btn, settings_btn, exit_btn]
            .spacing(20)
            .height(Length::FillPortion(6))
            .width(Length::Fill)
            .padding(Padding::from([40, 0]))
            .align_items(Alignment::Center)
            .into();

        let darkmode_toggle = toggler(Some("Dark Mode:".to_string()), darkmode, |checked| {
            let theme_mode: ThemeMode = checked
                .then_some(ThemeMode::Dark)
                .unwrap_or(ThemeMode::Light);
            AppMessage::ThemeChanged(theme_mode)
        })
        .text_size(11)
        .spacing(20)
        .text_alignment(alignment::Horizontal::Center);
        let version_info = text(format!("Version: {}", env!("CARGO_PKG_VERSION"))).size(11);
        let bottom_extra: Element<_> = column![darkmode_toggle, version_info]
            .spacing(10)
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .into();

        let content: Element<_> = column![brand, buttons, bottom_extra]
            .spacing(10)
            .padding(10)
            .into();

        container(content)
            .width(Length::Fixed(150.0))
            .height(Length::Fill)
            .style(theme::Container::Custom(Box::new(PanelTheme)))
            .into()
    }
}
