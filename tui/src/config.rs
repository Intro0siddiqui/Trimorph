use ratatui::style::Color;
use serde::de::{self, Deserializer};
use serde::Deserialize;
use std::{fs, path::PathBuf};

// Newtype wrapper for ratatui::style::Color to implement Deserialize
#[derive(Debug, Clone, Copy)]
pub struct SerializableColor(pub Color);

impl<'de> Deserialize<'de> for SerializableColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let color = match s.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" => Color::Gray,
            "darkgray" => Color::DarkGray,
            "lightred" => Color::LightRed,
            "lightgreen" => Color::LightGreen,
            "lightyellow" => Color::LightYellow,
            "lightblue" => Color::LightBlue,
            "lightmagenta" => Color::LightMagenta,
            "lightcyan" => Color::LightCyan,
            "white" => Color::White,
            _ => return Err(de::Error::custom(format!("unknown color: {}", s))),
        };
        Ok(SerializableColor(color))
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub theme: Theme,
}

#[derive(Debug, Deserialize)]
pub struct Theme {
    pub main_bg: SerializableColor,
    pub main_fg: SerializableColor,
    pub highlight_bg: SerializableColor,
    pub highlight_fg: SerializableColor,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme {
                main_bg: SerializableColor(Color::Black),
                main_fg: SerializableColor(Color::White),
                highlight_bg: SerializableColor(Color::Blue),
                highlight_fg: SerializableColor(Color::White),
            },
        }
    }
}

pub fn load_config() -> Config {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("trimorph")
        .join("tui.toml");

    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(config) = toml::from_str(&content) {
            return config;
        }
    }
    Config::default()
}