use crate::xlib;

use toml::Table;

use std::env;
use std::fs;


pub struct Config {
    pub foreground: xlib::Color,
    pub background: xlib::Color,
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
                foreground: xlib::Color::from_str(table["foreground"].as_str().unwrap_or_default())?,
                background: xlib::Color::from_str(table["background"].as_str().unwrap_or_default())?,
            })
        } else {
            println!("[+] no config found");

            Ok(Config {
                foreground: xlib::Color::new(255, 255, 255),
                background: xlib::Color::new(0, 0, 0),
            })
        }
    }
}

