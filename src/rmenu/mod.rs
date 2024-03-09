use crate::config::Config;
use crate::xlib;

use std::collections::HashMap;
use std::process::Command;
use std::env;
use std::fs;

struct Search {
    value: String,
    select: usize,
}

struct Xft {
    font: *mut x11::xft::XftFont,
    foreground: x11::xft::XftColor,
    background: x11::xft::XftColor,
}

pub struct Menu {
    display: xlib::Display,
    config: Config,
    search: Search,
    xft: Xft,

    applications: HashMap<String, String>,
    matches: Vec<(String, String)>,
    should_close: bool,
    capslock: bool,
}

impl Menu {
    pub fn new(config_path: &str) -> Result<Menu, Box<dyn std::error::Error>> {
        let config = Config::load(config_path)?;
        let mut display = xlib::Display::open(&config)?;

        let font = display.load_font(&config.font)?;
        let foreground = display.xft_color_alloc_name(config.foreground)?;
        let background = display.xft_color_alloc_name(config.background)?;

        Ok(Menu {
            display,
            config,
            search: Search {
                value: String::new(),
                select: 0,
            },
            xft: Xft {
                font,
                foreground,
                background,
            },

            applications: HashMap::new(),
            matches: vec![(String::new(), String::new()); 5],
            should_close: false,
            capslock: false,
        })
    }

    fn load_applications(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let paths = env::var("PATH")?;

        for path in paths.split(':') {
            for entry in fs::read_dir(path)? {
                let file = entry?;

                self.applications.insert(
                    file.file_name().to_str().unwrap_or_default().to_string(),
                    file.path().to_str().unwrap_or_default().to_string()
                );
            }
        }

        Ok(())
    }

    fn search(&mut self, capacity: usize) {
        let mut matches: Vec<(String, String)> = self.applications.iter()
            .filter(|(name, _)| name.contains(&self.search.value))
            .map(|(name, path)| (name.clone(), path.clone()))
            .collect();

        matches.sort_by_key(|(name, _)| name.clone());
        matches.resize(capacity, (String::new(), String::new()));

        self.matches = matches;
    }

    fn draw(&mut self) {
        self.display.clear_window();

        self.display.xft_draw_string(&self.search.value, 10, 15, self.xft.font, &self.xft.foreground);

        self.display.xft_draw_string(
            "rmenu v1.1",
            self.display.width - self.display.xft_measure_string("rmenu v1.1", self.xft.font).width as i32 - 10,
            15,
            self.xft.font,
            &self.xft.foreground
        );

        if !self.search.value.is_empty() {
            self.display.draw_rec(
                self.display.xft_measure_string(&self.search.value, self.xft.font).width as i32 + 10,
                0,
                8,
                20,
                xlib::Color::new(255, 255, 255)
            );
        } else {
            self.display.draw_rec(10, 0, 8, 20, xlib::Color::new(255, 255, 255));
        }

        let mut offset = 150;

        for (index, (name, _)) in self.matches.iter().enumerate() {
            if !name.is_empty() && index < self.display.width as usize {
                let width = self.display.xft_measure_string(&name, self.xft.font).width as i32;

                if index == self.search.select {
                    self.display.draw_rec(offset, 0, width as u32, 23, self.config.foreground);
                    self.display.xft_draw_string(&name, offset, 15, self.xft.font, &self.xft.background);
                } else {
                    self.display.xft_draw_string(&name, offset, 15, self.xft.font, &self.xft.foreground);
                }

                offset += width + 10;
            }
        }
    }

    fn reset_search(&mut self) {
        self.search.select = 0;
    }

    fn handle_event(&mut self, event: x11::xlib::XEvent) -> Result<(), Box<dyn std::error::Error>> {
        match unsafe { event.type_ } {
            x11::xlib::KeyPress => {
                match unsafe { self.display.keycode_to_keysym(event.key.keycode as u8) as u32 } {
                    x11::keysym::XK_Left => {
                        if self.search.select > 0 {
                            self.search.select -= 1;
                        }
                    },
                    x11::keysym::XK_Right => {
                        if self.search.select < self.matches.iter().filter(|(name, _)| !name.is_empty()).collect::<Vec<_>>().len() - 1 {
                            self.search.select += 1;
                        }
                    },
                    x11::keysym::XK_Escape => {
                        self.should_close = true;
                    },
                    x11::keysym::XK_Shift_L => {
                        self.capslock = true;
                    },
                    x11::keysym::XK_Return => {
                        let path = self.applications.get(&self.matches[self.search.select].0).ok_or::<Box<dyn std::error::Error>>("no such program".into())?;

                        Command::new(path).spawn()?;

                        self.should_close = true;
                    },
                    x11::keysym::XK_BackSpace => {
                        self.search.value.pop();

                        self.reset_search();
                    },
                    x11::keysym::XK_space => {
                        self.search.value.push(' ');

                        self.reset_search();
                    },
                    _ => {
                        let key = self.display.keycode_to_string(unsafe { event.key.keycode as u8 })?;
                        let character = key.chars().next().unwrap_or_default();

                        if self.capslock {
                            self.search.value.push(character.to_ascii_uppercase());
                        } else {
                            self.search.value.push(character);
                        }

                        self.reset_search();
                    },
                }
            },
            x11::xlib::KeyRelease => {
                match unsafe { self.display.keycode_to_keysym(event.key.keycode as u8) as u32 } {
                    x11::keysym::XK_Shift_L => {
                        self.capslock = false;
                    },
                    _ => {},
                }
            },
            _ => {},
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[+] running rmenu");

        self.load_applications()?;

        self.display.set_property("_NET_WM_WINDOW_TYPE", "_NET_WM_WINDOW_TYPE_DOCK")?;
        self.display.set_property("_NET_WM_STATE", "_NET_WM_STATE_ABOVE")?;
        self.display.set_property("_NET_WM_STATE", "_NET_WM_STATE_MODAL")?;

        self.display.map_window();

        while !self.should_close {
            self.display.select_input();

            if let Some(event) = self.display.poll_event() {
                self.handle_event(event)?;
                self.search(10);
                self.draw();
            }
        }

        Ok(())
    }
}


