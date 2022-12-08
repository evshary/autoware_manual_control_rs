mod manual_control;

use zenoh::prelude::sync::*;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use manual_control::ManualController;
use std::sync::Arc;

const MAX_STEER_ANGLE  : f32 = 0.3925; // 22.5 * (PI / 180)
const STEP_STEER_ANGLE : f32 = 0.0174; // 1 * (PI / 180)
const MAX_SPEED        : f32 = 27.78;  // 100 km/hr = 27.78 m/s
const STEP_SPEED       : f32 = 1.389;  // 5 km/hr = 1.389 m/s

fn print_help() {
    println!("------------------------------------");
    println!("| Different Mode:                  |");
    println!("|   z: Toggle auto & external mode |");
    println!("|   x: Gear Type => Drive          |");
    println!("|   c: Gear Type => Reverse        |");
    println!("|   v: Gear Type => Park           |");
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
    let mut velocity = 0.0;  // m/s
    let mut angle = 0.0;     // radian

    let z_session = Arc::new(zenoh::open(config::peer()).res().unwrap());
    let mut manual_controller = ManualController::new(z_session.clone());
    manual_controller.init(z_session.clone());
    print_help();
    crossterm::terminal::enable_raw_mode().unwrap();
    loop {
        match crossterm::event::read() {
            Ok(Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, kind: _, state: _})) => {
                break;
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('z'), modifiers: _, kind: _, state: _})) => {
                manual_controller.toggle_gate_mode();
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('x'), modifiers: _, kind: _, state: _})) => {
                manual_controller.pub_gear_command(2);
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('c'), modifiers: _, kind: _, state: _})) => {
                manual_controller.pub_gear_command(20);
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('v'), modifiers: _, kind: _, state: _})) => {
                manual_controller.pub_gear_command(22);
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('s'), modifiers: _, kind: _, state: _})) => {
                println!("{}\r", manual_controller.get_status());
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('u'), modifiers: _, kind: _, state: _})) => {
                velocity = num::clamp(velocity + STEP_SPEED, 0.0, MAX_SPEED);
                manual_controller.update_control_command(velocity, angle);
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('i'), modifiers: _, kind: _, state: _})) => {
                velocity = 0.0;
                manual_controller.update_control_command(velocity, angle);
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('o'), modifiers: _, kind: _, state: _})) => {
                velocity = num::clamp(velocity - STEP_SPEED, 0.0, MAX_SPEED);
                manual_controller.update_control_command(velocity, angle);
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('j'), modifiers: _, kind: _, state: _})) => {
                angle = num::clamp(angle + STEP_STEER_ANGLE, -MAX_STEER_ANGLE, MAX_STEER_ANGLE);
                manual_controller.update_control_command(velocity, angle);
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('k'), modifiers: _, kind: _, state: _})) => {
                angle = 0.0;
                manual_controller.update_control_command(velocity, angle);
            },
            Ok(Event::Key(KeyEvent {code: KeyCode::Char('l'), modifiers: _, kind: _, state: _})) => {
                angle = num::clamp(angle - STEP_STEER_ANGLE, -MAX_STEER_ANGLE, MAX_STEER_ANGLE);
                manual_controller.update_control_command(velocity, angle);
            },
            Ok(_) => {},
            Err(_) => {}
        }
    }
    crossterm::terminal::disable_raw_mode().unwrap();
}
