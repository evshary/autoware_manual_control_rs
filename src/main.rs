use zenoh::prelude::sync::*;
use zenoh::subscriber::Subscriber;
use zenoh::config::Config;
use zenoh::buffers::reader::HasReader;
use serde_derive::{Serialize, Deserialize};
use cdr::{CdrLe, Infinite};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

struct ManualController<'a> {
    session: Session,
    gate_mode_sub: Option<Subscriber<'a, ()>>,
}

impl<'a> ManualController<'a> {
    pub fn new() -> Self {
        ManualController {
            session: zenoh::open(Config::default()).res().unwrap(),
            gate_mode_sub: None
        }
    }

    pub fn init<'b: 'a>(&'b mut self) {
        let gate_mode_key = String::from("rt/control/current_gate_mode");
        self.gate_mode_sub = Some(self.session
            .declare_subscriber(gate_mode_key)
            .callback_mut(move |sample| {
                match cdr::deserialize_from::<_, GateMode, _>(sample.payload.reader(), cdr::size::Infinite) {
                    Ok(gatemode) => {
                        println!("gatemode.date={}\r", gatemode.data);
                    }
                    Err(_) => {}
                }
            })
            .res()
            .unwrap());
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
struct GateMode {
    data: u8,  // 0: AUTO, 1: EXTERNAL
}

fn print_help() {
    println!("------------------------------------");
    println!("| Different Mode:                  |");
    println!("|   z: Toggle auto & external mode |");
    println!("|   x: GateMode => Drive           |");
    println!("|   c: GateMode => Reverse         |");
    println!("|   v: GateMode => Park            |");
    println!("|   s: View current mode           |");
    println!("| Speed:                           |");
    println!("|   u: Increase speed              |");
    println!("|   i: Set speed to 0              |");
    println!("|   o: Decrease speed              |");
    println!("| Steering Angle                   |");
    println!("|   j: Left turn                   |");
    println!("|   k: Set angle to 0              |");
    println!("|   l: Right turn                  |");
    println!("------------------------------------");
}

fn main() {
    let mut manual_controller = ManualController::new();
    manual_controller.init();
    print_help();
    crossterm::terminal::enable_raw_mode().unwrap();
    loop {
        match crossterm::event::read() {
            Ok(Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, kind: _, state: _})) => {
                break;
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('z'), modifiers: _, kind: _, state: _})) => {
                // Do something
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('x'), modifiers: _, kind: _, state: _})) => {
                // Do something
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('c'), modifiers: _, kind: _, state: _})) => {
                // Do something
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('v'), modifiers: _, kind: _, state: _})) => {
                // Do something
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('s'), modifiers: _, kind: _, state: _})) => {
                // Do something
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('u'), modifiers: _, kind: _, state: _})) => {
                // Do something
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('i'), modifiers: _, kind: _, state: _})) => {
                // Do something
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('o'), modifiers: _, kind: _, state: _})) => {
                // Do something
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('j'), modifiers: _, kind: _, state: _})) => {
                // Do something
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('k'), modifiers: _, kind: _, state: _})) => {
                // Do something
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('l'), modifiers: _, kind: _, state: _})) => {
                // Do something
            },
            Ok(_) => {},
            Err(_) => {}
        }
    }
    crossterm::terminal::disable_raw_mode().unwrap();
}
