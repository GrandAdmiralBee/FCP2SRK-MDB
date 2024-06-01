use ratatui::style::Color;

const WARN: Color = Color::Yellow;
const INFO: Color = Color::White;
const ERROR: Color = Color::Red;
const COMMAND: Color = Color::LightGreen;

pub struct Logger {
    pub contents: Vec<(String, LogLevel)>,
}

impl Logger {
    pub fn log(&mut self, message: &str, level: LogLevel) {
        self.contents.push((message.to_string(), level));
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LogLevel {
    Warn,
    Info,
    Error,
    Trace,
}

impl LogLevel {
    pub fn get_color(&self) -> Color {
        match self {
            Self::Warn => WARN,
            Self::Error => ERROR,
            Self::Info => INFO,
            Self::Trace => COMMAND,
        }
    }
}
