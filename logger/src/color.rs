#[derive(Clone, Copy)]
pub enum Color {
    Black = 30,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

pub struct ColoredStr {
    content: String,
    bold: bool,
    bright: bool,
    underline: bool,
    color: Color,
}

impl ColoredStr {
    pub fn new() -> Self {
        ColoredStr {
            content: String::new(),
            bold: false,
            bright: false,
            underline: false,
            color: Color::White,
        }
    }

    pub fn content<'a>(&'a mut self, con: &str) -> &'a mut Self {
        self.content = con.to_string();
        self
    }

    pub fn bold(&mut self) -> &mut Self {
        self.bold = true;
        self
    }

    pub fn bright(&mut self) -> &mut Self {
        self.bright = true;
        self
    }

    pub fn underline(&mut self) -> &mut Self {
        self.underline = true;
        self
    }

    pub fn color(&mut self, co: Color) -> &mut Self {
        self.color = co;
        self
    }

    pub fn build(&self) -> String {
        let bright_flag: &str = if self.bright { ";1" } else { "" };
        let mut deco_flag: u8 = self.color as u8;
        if self.bold {
            deco_flag = 1_u8;
        }
        if self.underline {
            deco_flag = 4_u8;
        }
        format!("\x1b[{}{}m{}\x1b[0m", deco_flag, bright_flag, self.content)
    }
}

impl Default for ColoredStr {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for ColoredStr {
    fn from(s: &str) -> Self {
        ColoredStr {
            content: s.to_string(),
            ..Default::default()
        }
    }
}

impl From<String> for ColoredStr {
    fn from(s: String) -> Self {
        ColoredStr {
            content: s,
            ..Default::default()
        }
    }
}
