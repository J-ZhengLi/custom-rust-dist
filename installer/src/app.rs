//! The frontend proportion of this application
//!
//! The main interface of this application consisting 2 major parts:
//! - A side panel contains static widgets. [`side_panel`]
//! - Main frame showing different contents. [`main_frame`]

mod main_frame;
mod side_panel;
mod themes;

use iced::theme::Theme;
use iced::widget::{container, row};
use iced::{executor, window, Application, Color, Command, Element, Length, Settings};
use side_panel::SidePanel;

#[derive(Debug, Clone)]
pub(crate) enum AppMessage {
    ThemeChanged(ThemeMode),
    ShowToolchains,
    ShowSettings,
    Exit,
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
        "GUI for rustup".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<AppMessage> {
        match message {
            AppMessage::ThemeChanged(tm) => self.theme = tm.into(),
            AppMessage::Exit => {
                return window::close();
            }
            _ => {
                println!("{message:?} message recived, but it's not implemented");
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let side_panel = SidePanel::default();
        let content: Element<_> = row![side_panel.view(self.theme == Theme::Dark)].into();

        container(if self.debug {
            content.explain(Color::BLACK)
        } else {
            content
        })
        .width(Length::Fill)
        .height(Length::Fill)
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
