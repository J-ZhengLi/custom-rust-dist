use super::{AppMessage, ThemeMode};
use iced::theme::Button;
use iced::widget::{button, column, container, image, radio, Container};
use iced::{Element, Length, Theme};

#[derive(Debug, Clone)]
pub(crate) enum SidePanelMsg {
    ThemeChanged(ThemeMode),
    Exit,
}

impl From<SidePanelMsg> for AppMessage {
    fn from(value: SidePanelMsg) -> Self {
        AppMessage::SidePanel(value)
    }
}

#[derive(Debug, Default)]
pub(crate) struct SidePanel {
    theme: Theme,
}

impl<'a> SidePanel {
    pub fn view(&self, width: u16) -> Container<'a, AppMessage> {
        let logo =
            container(image(format!("{}/res/logo.png", env!("CARGO_MANIFEST_DIR"))).width(width))
                .width(Length::Fill)
                .center_x();
        let choose_theme = [ThemeMode::Light, ThemeMode::Dark].iter().fold(
            column![].spacing(10),
            |column, theme| {
                column.push(radio(
                    format!("{theme:?}"),
                    *theme,
                    Some(match self.theme {
                        Theme::Dark => ThemeMode::Dark,
                        _ => ThemeMode::Light,
                    }),
                    set_theme,
                ))
            },
        );
        let exit_btn = button("exit")
            .style(Button::Secondary)
            .width(100)
            .padding(10)
            .on_press(AppMessage::from(SidePanelMsg::Exit));
        let content: Element<_> = column![logo, choose_theme, exit_btn,]
            .spacing(20)
            .padding(20)
            .into();

        container(content)
            .width(Length::Fixed(150.0))
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

fn set_theme(theme: ThemeMode) -> AppMessage {
    SidePanelMsg::ThemeChanged(theme).into()
}
