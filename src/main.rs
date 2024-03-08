mod config;
mod rmenu;
mod xlib;

use rmenu::Menu;

use std::process;
use std::env;

const HELP: &str = "
Usage: rmenu [options]
options:
    -config <path>      custom config path
    -help               show help
";

pub struct Args {
    config_path: String,
}

impl Args {
    pub fn parse() -> Result<Args, Box<dyn std::error::Error>> {
        let mut argc = 0;
        let mut args = Args {
            config_path: String::new(),
        };

        let argv = env::args().collect::<Vec<String>>();

        while argc < argv.len() {
            match argv[argc].as_str() {
                "-help" => {
                    return Err(HELP.into());
                },
                "-config" => {
                    argc += 1;

                    if let Some(path) = argv.get(argc) {
                        args.config_path = path.clone();
                    } else {
                        return Err("Usage: -config <path>".into());
                    }
                },
                _ => {},
            }

            argc += 1;
        }

        Ok(args)
    }
}

fn main() {
    let args = match Args::parse() {
        Ok(args) => args,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        },
    };

    let mut menu = match Menu::new(&args.config_path) {
        Ok(menu) => menu,
        Err(err) => {
            println!("[ERROR] failed to create menu: {}", err);
            process::exit(1);
        },
    };

    if let Err(err) = menu.run() {
        println!("[ERROR] failed to run: {}", err);
        process::exit(1);
    }
}

