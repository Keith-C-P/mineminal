// Colors (Night Mode)
// BG: #384048
// Killed By: #ee6666
// FG: #485058
// Flag: #f76161
// 1: #7cc7ff
// 2: #66c266
// 3: #ff7788
// 4: #ee88ff
// 5: #ddaa22
// 6: #66cccc
// 7: #999999
// 8: #d0d8df

use log::{debug, info};
use ratatui::style::{Color, Style};
use serde::Deserialize;
use std::fs;

fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim_start_matches('#');

    let r = u8::from_str_radix(&s[0..2], 16).map_err(serde::de::Error::custom)?;
    let g = u8::from_str_radix(&s[2..4], 16).map_err(serde::de::Error::custom)?;
    let b = u8::from_str_radix(&s[4..6], 16).map_err(serde::de::Error::custom)?;

    Ok(Color::Rgb(r, g, b))
}

#[derive(Deserialize)]
pub struct Colors {
    #[serde(deserialize_with = "deserialize_color")]
    background: Color,
    #[serde(deserialize_with = "deserialize_color")]
    killed_by_background: Color,
    #[serde(deserialize_with = "deserialize_color")]
    foreground: Color,
    #[serde(deserialize_with = "deserialize_color")]
    flag: Color,
    #[serde(deserialize_with = "deserialize_color")]
    bomb: Color,

    #[serde(deserialize_with = "deserialize_color")]
    one: Color,
    #[serde(deserialize_with = "deserialize_color")]
    two: Color,
    #[serde(deserialize_with = "deserialize_color")]
    three: Color,
    #[serde(deserialize_with = "deserialize_color")]
    four: Color,
    #[serde(deserialize_with = "deserialize_color")]
    five: Color,
    #[serde(deserialize_with = "deserialize_color")]
    six: Color,
    #[serde(deserialize_with = "deserialize_color")]
    seven: Color,
    #[serde(deserialize_with = "deserialize_color")]
    eight: Color,
}

impl Colors {
    pub fn from_file(path: &str) -> Self {
        let raw_settings =
            fs::read_to_string(path).expect("Should have been able to read the file");
        info!("{}", raw_settings);

        #[derive(Deserialize)]
        struct Settings {
            color: Colors,
        }

        let settings: Settings = toml::from_str(&raw_settings).expect("Failed to parse TOML");
        settings.color
    }

    pub fn style(&self, fg: Color) -> Style {
        Style::new().bg(self.background).fg(fg)
    }

    pub fn kill_style(&self, fg: Color) -> Style {
        Style::new().bg(self.killed_by_background).fg(fg)
    }
    pub fn background(&self) -> Style {
        Style::new().bg(self.background)
    }

    pub fn number(&self, n: u8) -> Style {
        assert!(n < 9);
        match n {
            1 => self.style(self.one),
            2 => self.style(self.two),
            3 => self.style(self.three),
            4 => self.style(self.four),
            5 => self.style(self.five),
            6 => self.style(self.six),
            7 => self.style(self.seven),
            8 => self.style(self.eight),
            _ => self.style(self.foreground),
        }
    }

    pub fn flag(&self) -> Style {
        self.style(self.flag)
    }

    pub fn bomb(&self) -> Style {
        self.style(self.bomb)
    }

    pub fn kill_flag(&self) -> Style {
        self.style(self.killed_by_background).reversed()
    }

    pub fn kill_bomb(&self) -> Style {
        self.kill_style(self.bomb)
    }

    pub fn foreground(&self) -> Style {
        self.style(self.foreground)
    }
}
