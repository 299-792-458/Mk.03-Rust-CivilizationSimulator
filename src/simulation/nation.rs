use colored::Color as ColoredColor;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Nation {
    Tera,
    Sora,
    Aqua,
    Solar,
    Luna,
}

impl Nation {
    pub fn name(&self) -> &'static str {
        match self {
            Nation::Tera => "Tera",
            Nation::Sora => "Sora",
            Nation::Aqua => "Aqua",
            Nation::Solar => "Solar",
            Nation::Luna => "Luna",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Nation::Tera => Color::Blue,
            Nation::Sora => Color::Red,
            Nation::Aqua => Color::Green,
            Nation::Solar => Color::Yellow,
            Nation::Luna => Color::White,
        }
    }

    pub fn logging_color(&self) -> ColoredColor {
        match self {
            Nation::Tera => ColoredColor::Blue,
            Nation::Sora => ColoredColor::Red,
            Nation::Aqua => ColoredColor::Green,
            Nation::Solar => ColoredColor::Yellow,
            Nation::Luna => ColoredColor::White,
        }
    }
}
