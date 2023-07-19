//! The frontend proportion of this application

mod side_panel;

use iced::theme::Theme;
use iced::widget::{container, row, vertical_rule};
use iced::{executor, window, Application, Color, Command, Element, Length, Settings};
use side_panel::{SidePanel, SidePanelMsg};

#[derive(Debug, Clone)]
pub(crate) enum AppMessage {
    SidePanel(SidePanelMsg),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ThemeMode {
    Light,
    Dark,
}

impl From<ThemeMode> for Theme {
    fn from(value: ThemeMode) -> Self {
        match value {
            ThemeMode::Dark => Theme::Dark,
            ThemeMode::Light => Theme::Light,
        }
    }
}

#[derive(Default)]
pub(crate) struct App {
    theme: Theme,
    debug: bool,
}

impl Application for App {
    type Message = AppMessage;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<AppMessage>) {
        #[cfg(feature = "debug")]
        let debug = true;
        #[cfg(not(feature = "debug"))]
        let debug = false;

        let app = Self {
            debug,
            ..Default::default()
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        "Rustup GUI".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<AppMessage> {
        match message {
            AppMessage::SidePanel(msg) => {
                match msg {
                    SidePanelMsg::ThemeChanged(tm) => {
                        self.theme = tm.into();
                    }
                    SidePanelMsg::Exit => {
                        return window::close();
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let side_panel = SidePanel::default();

        let content: Element<_> = row![side_panel.view(100), vertical_rule(10),]
            .spacing(20)
            .padding(20)
            .into();

        container(if self.debug {
            content.explain(Color::BLACK)
        } else {
            content
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

pub(crate) fn default_settings() -> Settings<()> {
    Settings {
        window: window::Settings {
            size: (800, 540),
            ..Default::default()
        },
        ..Default::default()
    }
}
