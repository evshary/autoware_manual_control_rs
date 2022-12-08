use zenoh::prelude::sync::*;
use zenoh::subscriber::Subscriber;
use zenoh::publication::Publisher;
use zenoh::buffers::reader::HasReader;
use serde_derive::{Serialize, Deserialize};
use cdr::{CdrLe, Infinite};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use atomic_float::{AtomicF32};
use std::thread;
use std::time::Duration;

pub struct ManualController<'a> {
    // publisher
    publisher_gate_mode: Publisher<'a>,
    client_engage_req: Publisher<'a>,
    publisher_gear_command: Publisher<'a>,
    // subscriber
    _subscriber_gate_mode: Option<Subscriber<'a, ()>>,
    _subscriber_engage: Option<Subscriber<'a, ()>>,
    _subscriber_gear_command: Option<Subscriber<'a, ()>>,
    _subscriber_velocity: Option<Subscriber<'a, ()>>,
    // settings
    steering_tire_angle: Arc<AtomicF32>,
    target_velocity: Arc<AtomicF32>,
    // status
    gate_mode: Arc<AtomicU8>,
    current_engage: Arc<AtomicBool>,
    gear_command: Arc<AtomicU8>,
    current_velocity: Arc<AtomicF32>,
}

impl<'a> ManualController<'a> {
    pub fn new(z_session: Arc<Session>) -> Self {
        let publisher_gate_mode = z_session
            .declare_publisher("rt/control/gate_mode_cmd")
            .res()
            .unwrap();
        let client_engage_req = z_session
            .declare_publisher("rq/api/autoware/set/engageRequest")
            .res()
            .unwrap();
        let publisher_gear_command = z_session
            .declare_publisher("rt/external/selected/gear_cmd")
            .res()
            .unwrap();

        ManualController {
            // publisher
            publisher_gate_mode,
            client_engage_req,
            publisher_gear_command,
            // subscriber
            _subscriber_gate_mode: None,
            _subscriber_engage: None,
            _subscriber_gear_command: None,
            _subscriber_velocity: None,
            // settings
            steering_tire_angle: Arc::new(AtomicF32::new(0.0)),
            target_velocity: Arc::new(AtomicF32::new(0.0)),
            // status
            gate_mode: Arc::new(AtomicU8::new(0)),
            current_engage: Arc::new(AtomicBool::new(false)),
            gear_command: Arc::new(AtomicU8::new(0)),
            current_velocity: Arc::new(AtomicF32::new(0.0)),
        }
    }

    pub fn init(&mut self, z_session: Arc<Session>) {
        let gate_mode = self.gate_mode.clone();
        self._subscriber_gate_mode = Some(z_session
            .declare_subscriber("rt/control/current_gate_mode")
            .callback_mut(move |sample| {
                match cdr::deserialize_from::<_, GateMode, _>(sample.payload.reader(), cdr::size::Infinite) {
                    Ok(gatemode) => {
                        //println!("gatemode.date={}\r", gatemode.data);
                        gate_mode.store(gatemode.data, Ordering::Relaxed);
                    },
                    Err(_) => {},
                }
            })
            .res()
            .unwrap());
        let current_engage = self.current_engage.clone();
        self._subscriber_engage = Some(z_session
            .declare_subscriber("rt/api/autoware/get/engage")
            .callback_mut(move |sample| {
                match cdr::deserialize_from::<_, GetEngage, _>(sample.payload.reader(), cdr::size::Infinite) {
                    Ok(engage) => {
                        //println!("Engage: {}\r", engage.enable);
                        current_engage.store(engage.enable, Ordering::Relaxed);
                    },
                    Err(_) => {},
                }
            })
            .res()
            .unwrap());
        let gear_cmd = self.gear_command.clone();
        self._subscriber_gear_command = Some(z_session
            .declare_subscriber("rt/vehicle/status/gear_status")
            .callback_mut(move |sample| {
                match cdr::deserialize_from::<_, GearCommand, _>(sample.payload.reader(), cdr::size::Infinite) {
                    Ok(gearcmd) => {
                        //println!("GearCommand: {}\r", gearcmd.command);
                        gear_cmd.store(gearcmd.command, Ordering::Relaxed);
                    },
                    Err(_) => {},
                }
            })
            .res()
            .unwrap());
        let current_velocity = self.current_velocity.clone();
        self._subscriber_velocity = Some(z_session
            .declare_subscriber("rt/vehicle/status/velocity_status")
            .callback_mut(move |sample| {
                match cdr::deserialize_from::<_, CurrentVelocity, _>(sample.payload.reader(), cdr::size::Infinite) {
                    Ok(velocity) => {
                        //println!("Velocity: {}\r", velocity.longitudinal_velocity);
                        current_velocity.store(velocity.longitudinal_velocity, Ordering::Relaxed);
                    },
                    Err(_) => {},
                }
            })
            .res()
            .unwrap());
        
        let steering_tire_angle = self.steering_tire_angle.clone();
        let target_velocity = self.target_velocity.clone();
        let gear_cmd = self.gear_command.clone();
        let current_velocity = self.current_velocity.clone();
        let publisher_control_command = Arc::new(z_session
            .declare_publisher("rt/external/selected/control_cmd")
            .res()
            .unwrap());
        thread::spawn(move || { loop {
            //println!("v:{} angle:{}\r", target_velocity.load(Ordering::Relaxed),
            //                            steering_tire_angle.load(Ordering::Relaxed));
            let real_target_velocity = target_velocity.load(Ordering::Relaxed) *
                                       (if gear_cmd.load(Ordering::Relaxed) == 2 { 1.0 } else { -1.0 });
            let acceleration = num::clamp(target_velocity.load(Ordering::Relaxed) - current_velocity.load(Ordering::Relaxed).abs(), -1.0, 1.0);
            let empty_time = TimeStamp { sec: 0, nsec: 0 };
            let control_cmd = AckermannControlCommand {
                ts: empty_time.clone(),
                lateral: AckermannLateralCommand { 
                    ts: empty_time.clone(),
                    steering_tire_angle: steering_tire_angle.load(Ordering::Relaxed),
                    steering_tire_rotation_rate: 0.0,
                },
                longitudinal: LongitudinalCommand {
                    ts: empty_time.clone(),
                    speed: real_target_velocity,
                    acceleration,
                    jerk: 0.0,
                },
            };
            let encoded = cdr::serialize::<_, _, CdrLe>(&control_cmd, Infinite).unwrap();
            publisher_control_command.put(encoded).res().unwrap();
            thread::sleep(Duration::from_millis(33));
        }});
    }

    fn pub_gate_mode(&self, mode: u8) {
        let gate_mode_data = GateMode { data: mode };
        let encoded = cdr::serialize::<_, _, CdrLe>(&gate_mode_data, Infinite).unwrap();
        self.publisher_gate_mode.put(encoded).res().unwrap();
    }

    fn send_client_engage(&self) {
        // TODO: We assign GUID and seq to 0, but this should be filled with meaningful value.
        let engage_data = Engage { header: ServiceHeader { guid: 0, seq: 0 }, enable: true };
        let encoded = cdr::serialize::<_, _, CdrLe>(&engage_data, Infinite).unwrap();
        self.client_engage_req.put(encoded).res().unwrap();
    }

    pub fn toggle_gate_mode(&self) -> bool {  // Return whether switch to external or not
        if self.gate_mode.load(Ordering::Relaxed) == 0 { // Auto
            self.pub_gate_mode(1);
            self.send_client_engage();
            true
        } else { // External
            self.pub_gate_mode(0);
            false
        }
    }

    pub fn pub_gear_command(&self, command: u8) {
        let gear_command = GearCommand { ts: TimeStamp { sec: 0, nsec: 0 }, command: command };
        let encoded = cdr::serialize::<_, _, CdrLe>(&gear_command, Infinite).unwrap();
        self.publisher_gear_command.put(encoded).res().unwrap();
    }

    pub fn update_control_command(&self, velocity: f32, angle: f32) {
        self.steering_tire_angle.store(angle, Ordering::Relaxed);
        self.target_velocity.store(velocity, Ordering::Relaxed);
    }

    pub fn get_status(&self) -> String {
        let mut s = String::from("Enage:");
        s += if self.current_engage.load(Ordering::Relaxed) { "Ready" } else {"Not Ready"};
        s += "\tGate Mode:";
        s += match self.gate_mode.load(Ordering::Relaxed) {
            0 => "Auto",
            1 => "External",
            _ => "Unknown",
        };
        s += "\tGear:";
        s += match self.gear_command.load(Ordering::Relaxed) {
            2 => "D",
            20 => "R",
            22 => "P",
            23 => "L",
            _ => "?",
        };
        s
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
struct TimeStamp {
    sec: i32,
    nsec: u32,
}

#[derive(Serialize, Deserialize, PartialEq)]
struct ServiceHeader {
    guid: i64,
    seq: u64,
}

#[derive(Serialize, Deserialize, PartialEq)]
struct GateMode {
    data: u8,  // 0: AUTO, 1: EXTERNAL
}

#[derive(Serialize, Deserialize, PartialEq)]
struct GetEngage {
    ts: TimeStamp,
    enable: bool,
}

#[derive(Serialize, Deserialize, PartialEq)]
struct Engage {
    header: ServiceHeader,
    enable: bool,
}

/* We don't need to get service response currently
#[derive(Serialize, Deserialize, PartialEq)]
struct ResponseStatus {
    header: ServiceHeader,
    code: u32,
    message: String,
}
*/

#[derive(Serialize, Deserialize, PartialEq)]
struct GearCommand {
    ts: TimeStamp,
    command: u8,
    // DRIVE = 2;
    // REVERSE = 20;
    // PARK = 22;
    // LOW = 23;
}

#[derive(Serialize, Deserialize, PartialEq)]
struct StdMsgsHeader {
    ts: TimeStamp,
    frameid: String,
}

#[derive(Serialize, Deserialize, PartialEq)]
struct CurrentVelocity {
    header: StdMsgsHeader,
    longitudinal_velocity: f32,
    lateral_velocity: f32,
    heading_rate: f32,
}

#[derive(Serialize, Deserialize, PartialEq)]
struct AckermannLateralCommand {
    ts: TimeStamp,
    steering_tire_angle: f32,
    steering_tire_rotation_rate: f32,
}

#[derive(Serialize, Deserialize, PartialEq)]
struct LongitudinalCommand {
    ts: TimeStamp,
    speed: f32,
    acceleration: f32,
    jerk: f32,
}

#[derive(Serialize, Deserialize, PartialEq)]
struct AckermannControlCommand {
    ts: TimeStamp,
    lateral: AckermannLateralCommand,
    longitudinal: LongitudinalCommand, 
}
