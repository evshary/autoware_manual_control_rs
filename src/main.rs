mod manual_control;

use clap::{App, Arg};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::f32::consts;
use std::sync::Arc;
use zenoh::prelude::sync::*;
use zenoh_ros_type::autoware_auto_vehicle_msgs;

use manual_control::ManualController;

const MAX_STEER_ANGLE: f32 = 0.3925; // 22.5 * (PI / 180)
const STEP_STEER_ANGLE: f32 = 0.0174; // 1 * (PI / 180)
const MAX_SPEED: f32 = 27.78; // 100 km/hr = 27.78 m/s
const STEP_SPEED: f32 = 1.389; // 5 km/hr = 1.389 m/s

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

fn parse_args() -> (Config, String) {
    let app = App::new("Autoware keyboard controller with Zenoh")
        .version("0.1.0")
        .arg(Arg::from_usage(
r#"-c, --config=[FILE] \
'The configuration file. Currently, this file must be a valid JSON5 file.'"#,
            ))
        .arg(Arg::from_usage(
r#"-l, --listen=[ENDPOINT]... \
'A locator on which this router will listen for incoming sessions.
Repeat this option to open several listeners.'"#,
                ),
            )
        .arg(Arg::from_usage(
r#"-s, --scope=[String]   'A string added as prefix to all routed DDS topics when mapped to a zenoh resource. This should be used to avoid conflicts when several distinct DDS systems using the same topics names are routed via zenoh'"#
            ));
    let args = app.get_matches();
    let mut config = match args.value_of("config") {
        Some(conf_file) => Config::from_file(conf_file).unwrap(),
        None => Config::default(),
    };
    if let Some(endpoints) = args.values_of("listen") {
        config
            .listen
            .endpoints
            .extend(endpoints.map(|p| p.parse().unwrap()))
    }
    let scope = match args.value_of("scope") {
        Some(s) => s.to_string() + "/",
        None => String::from(""),
    };
    (config, scope)
}

fn main() {
    let mut velocity = 0.0; // m/s
    let mut angle = 0.0; // radian

    let (config, scope) = parse_args();
    let z_session = Arc::new(zenoh::open(config).res().unwrap());
    let mut manual_controller = ManualController::new(z_session.clone(), scope);
    manual_controller.init(z_session.clone());
    print_help();
    crossterm::terminal::enable_raw_mode().unwrap();
    loop {
        match crossterm::event::read() {
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: _,
                state: _,
            })) => {
                break;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('z'),
                modifiers: _,
                kind: _,
                state: _,
            })) => {
                let new_mode = if manual_controller.toggle_gate_mode() {
                    "EXTERNAL"
                } else {
                    "AUTO"
                };
                println!("Toggle to {}\r", new_mode);
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('x'),
                modifiers: _,
                kind: _,
                state: _,
            })) => {
                manual_controller.pub_gear_command(autoware_auto_vehicle_msgs::gear_command::DRIVE);
                println!("Switch to DRIVE mode\r");
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: _,
                kind: _,
                state: _,
            })) => {
                manual_controller
                    .pub_gear_command(autoware_auto_vehicle_msgs::gear_command::REVERSE);
                println!("Switch to REVERSE mode\r");
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('v'),
                modifiers: _,
                kind: _,
                state: _,
            })) => {
                manual_controller.pub_gear_command(autoware_auto_vehicle_msgs::gear_command::PARK);
                println!("Switch to PARK mode\r");
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: _,
                kind: _,
                state: _,
            })) => {
                println!("{}\r", manual_controller.get_status());
            }
            Ok(Event::Key(KeyEvent {
                code: c,
                modifiers: _,
                kind: _,
                state: _,
            })) => {
                match c {
                    KeyCode::Char('u') => {
                        velocity = num::clamp(velocity + STEP_SPEED, 0.0, MAX_SPEED)
                    }
                    KeyCode::Char('i') => velocity = 0.0,
                    KeyCode::Char('o') => {
                        velocity = num::clamp(velocity - STEP_SPEED, 0.0, MAX_SPEED)
                    }
                    KeyCode::Char('j') => {
                        angle =
                            num::clamp(angle + STEP_STEER_ANGLE, -MAX_STEER_ANGLE, MAX_STEER_ANGLE)
                    }
                    KeyCode::Char('k') => angle = 0.0,
                    KeyCode::Char('l') => {
                        angle =
                            num::clamp(angle - STEP_STEER_ANGLE, -MAX_STEER_ANGLE, MAX_STEER_ANGLE)
                    }
                    _ => {}
                }
                manual_controller.update_control_command(velocity, angle);
                println!(
                    "angle(deg):{}\tvelocity(km/hr):{}\r",
                    (angle * 180.0 / consts::PI),
                    (velocity * 3600.0 / 1000.0)
                );
            }
            _ => {}
        }
    }
    crossterm::terminal::disable_raw_mode().unwrap();
}
