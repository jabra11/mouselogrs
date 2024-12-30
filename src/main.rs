use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};

use clap::Parser;
use input_linux_sys::{input_event, *};

mod db;
use db::Database;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path_to_mouse: String,

    #[arg(short, long, default_value_t = 800)]
    dpi: i32,

    #[arg(long("db"), default_value_t=String::from("/home/joerg/.MouseStats/mouse.json"))]
    path_to_db: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    println!("Selected {}", args.path_to_mouse);

    let mouse = File::open(args.path_to_mouse)?;
    let mut mouse = BufReader::new(mouse);

    const INPUTDATASIZE: usize = size_of::<input_event>();
    let mut buf: [u8; INPUTDATASIZE] = [0; INPUTDATASIZE];

    let evts = HashMap::from([(EV_REL, "EV_REL"), (EV_KEY, "EV_KEY")]);
    let evtkeys: Vec<i32> = evts.keys().cloned().collect();

    let keycodes = HashMap::from([
        (BTN_LEFT, "Left"),
        (BTN_RIGHT, "Right"),
        (BTN_MIDDLE, "Middle"),
        (BTN_SIDE, "Side"),
        (BTN_EXTRA, "Extra"),
        (REL_X, "RelX"),
        (REL_Y, "RelY"),
        (REL_WHEEL, "REL_WHEEL"),
    ]);
    let keycodekeys: Vec<i32> = keycodes.keys().cloned().collect();

    let mut db = Database::new(&args.path_to_db);

    loop {
        match mouse.read_exact(&mut buf) {
            Ok(_) => {
                let ie: input_event = unsafe { std::ptr::read(buf.as_ptr() as *const input_event) };

                if !(evtkeys.contains(&(ie.type_.into()))
                    && keycodekeys.contains(&(ie.code.into())))
                    // ignore release events
                    || ie.type_ as i32 == EV_KEY && ie.value == 0
                {
                    continue;
                }
                db.modify(ie.type_ as i32, ie.code as i32, ie.value as i64);
            }
            Err(e) => println!("got err {e:?}"),
        }
    }
}
