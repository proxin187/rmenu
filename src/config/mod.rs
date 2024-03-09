use crate::xlib;

use toml::Table;

use std::env;
use std::fs;


pub struct Config {
    pub foreground: xlib::Color,
    pub background: xlib::Color,
    pub font: String,
}

impl Config {
    pub fn load(config_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let home = env::var("HOME")?;

        let path = if config_path.is_empty() {
            format!("{}/.config/rmenu/config.toml", home)
        } else {
            config_path.to_string()
        };

        if let Ok(content) = fs::read_to_string(path) {
            println!("[+] loading config");

            let table = content.parse::<Table>()?;

            Ok(Config {
                foreground: xlib::Color::from_str(&Self::get_str(&table, "foreground").unwrap_or_default())?,
                background: xlib::Color::from_str(&Self::get_str(&table, "background").unwrap_or_default())?,
                font: Self::get_str(&table, "font").unwrap_or("DejaVu Sans Mono:size=11:antialias=true".to_string()),
            })
        } else {
            println!("[+] no config found");

            Ok(Config {
                foreground: xlib::Color::new(255, 255, 255),
                background: xlib::Color::new(0, 0, 0),
                font: String::from("DejaVu Sans Mono:size=11:antialias=true"),
            })
        }
    }

    fn get_str(table: &toml::map::Map<String, toml::Value>, key: &str) -> Option<String> {
        if let Some(value) = table.get(key) {
            if let Some(string) = value.as_str() {
                Some(string.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
}


