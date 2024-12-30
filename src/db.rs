use input_linux_sys::*;
use serde::{Deserialize, Serialize};
use serde_json::error::Category;
use std::time::Instant;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Data {
    // unit are clicks
    left: i64,
    right: i64,
    middle: i64,
    side: i64,
    extra: i64,
    wheel_up: i64,
    wheel_down: i64,

    // unit is are dots (as in DPI)
    swipe_right: i64,
    swipe_left: i64,
    swipe_up: i64,
    swipe_down: i64,
}

pub struct Database {
    pub data: Data,
    path: String,
    last_export: Option<Instant>,
}

impl Database {
    pub fn new(path: &str) -> Self {
        let json = match std::fs::exists(path) {
            Ok(true) => {
                println!("Read file at {path}");
                std::fs::read_to_string(path).unwrap()
            }
            Ok(false) => {
                eprintln!("No file found at {path}");
                eprintln!("Aborting!");
                panic!();
            }
            Err(e) => {
                eprintln!("Error reading {path}: {e:?}");
                eprintln!("Aborting!");
                panic!();
            }
        };

        let data: Data = match serde_json::from_str(&json) {
            Ok(d) => {
                println!("Imported data from {path}");
                d
            }
            Err(e) => {
                if e.classify() == Category::Data {
                    eprintln!("Parsing error: {e:?}");
                }
                println!("Starting over with fresh data");
                Data {
                    ..Default::default()
                }
            }
        };
        Database {
            data,
            path: String::from(path),
            last_export: None,
        }
    }

    pub fn modify(&mut self, type_: i32, code: i32, value: i64) {
        let d = &mut self.data;
        if type_ == EV_REL {
            match code {
                REL_X => {
                    if value > 0 {
                        d.swipe_right += value.abs();
                    } else {
                        d.swipe_left += value.abs();
                    }
                }
                REL_Y => {
                    if value > 0 {
                        d.swipe_down += value.abs();
                    } else {
                        d.swipe_up += value.abs();
                    }
                }
                REL_WHEEL => {
                    if value > 0 {
                        d.wheel_up += 1;
                    } else {
                        d.wheel_down += 1;
                    }
                }
                _ => unimplemented!(),
            }
        } else if type_ == EV_KEY {
            match code {
                BTN_LEFT => d.left += 1,
                BTN_RIGHT => d.right += 1,
                BTN_MIDDLE => d.middle += 1,
                BTN_SIDE => d.side += 1,
                BTN_EXTRA => d.extra += 1,
                _ => unimplemented!(),
            }
        }

        // export maybe
        if self.last_export.is_none_or(|exp| exp.elapsed().as_millis() >= 100) {
            self.export();
            self.last_export = Some(Instant::now());
        }
    }

    pub fn export(&self) {
        println!("Exporting data to {}", self.path);
        let json = serde_json::to_string_pretty(&self.data).unwrap();
        std::fs::write(&self.path, json).unwrap();
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        self.export();
    }
}
