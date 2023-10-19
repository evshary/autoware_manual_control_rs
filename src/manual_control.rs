use atomic_float::AtomicF32;
use cdr::{CdrLe, Infinite};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use zenoh::prelude::sync::*;
use zenoh::publication::Publisher;
use zenoh::subscriber::Subscriber;
use zenoh_ros_type::{
    autoware_auto_control_msgs, autoware_auto_vehicle_msgs, builtin_interfaces, service,
    tier4_control_msgs,
};

pub struct ManualController<'a> {
    // mode
    ros2: bool,
    // prefix
    prefix: String,
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
    pub fn new(z_session: Arc<Session>, ros2: bool, prefix: String) -> Self {
        let prefix_rt = prefix.clone() + if ros2 { "" } else { "rt/" };
        let prefix_rq = prefix.clone() + if ros2 { "" } else { "rq/" };
        let key_gate_mode = prefix_rt.clone() + "control/gate_mode_cmd";
        let key_client_engage = prefix_rq.clone() + "api/autoware/set/engageRequest";
        let key_gear_command = prefix_rt.clone() + "external/selected/gear_cmd";

        let publisher_gate_mode = z_session.declare_publisher(key_gate_mode).res().unwrap();
        let client_engage_req = z_session
            .declare_publisher(key_client_engage)
            .res()
            .unwrap();
        let publisher_gear_command = z_session.declare_publisher(key_gear_command).res().unwrap();

        ManualController {
            // mode
            ros2,
            // prefix
            prefix,
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
        let prefix_rt = self.prefix.clone() + if self.ros2 { "" } else { "rt/" };
        let key_gate_mode = prefix_rt.clone() + "control/current_gate_mode";
        let key_engage = prefix_rt.clone() + "api/autoware/get/engage";
        let key_gear_command = prefix_rt.clone() + "vehicle/status/gear_status";
        let key_velocity = prefix_rt.clone() + "vehicle/status/velocity_status";
        let key_control_command = prefix_rt.clone() + "external/selected/control_cmd";

        let gate_mode = self.gate_mode.clone();
        self._subscriber_gate_mode = Some(
            z_session
                .declare_subscriber(key_gate_mode)
                .callback_mut(move |sample| {
                    match cdr::deserialize_from::<_, tier4_control_msgs::GateMode, _>(
                        &*sample.payload.contiguous(),
                        cdr::size::Infinite,
                    ) {
                        Ok(gatemode) => {
                            //println!("gatemode.date={}\r", gatemode.data);
                            gate_mode.store(gatemode.data, Ordering::Relaxed);
                        }
                        Err(_) => {}
                    }
                })
                .res()
                .unwrap(),
        );
        let current_engage = self.current_engage.clone();
        self._subscriber_engage = Some(
            z_session
                .declare_subscriber(key_engage)
                .callback_mut(move |sample| {
                    match cdr::deserialize_from::<_, autoware_auto_vehicle_msgs::Engage, _>(
                        &*sample.payload.contiguous(),
                        cdr::size::Infinite,
                    ) {
                        Ok(engage) => {
                            //println!("Engage: {}\r", engage.enable);
                            current_engage.store(engage.enable, Ordering::Relaxed);
                        }
                        Err(_) => {}
                    }
                })
                .res()
                .unwrap(),
        );
        let gear_cmd = self.gear_command.clone();
        self._subscriber_gear_command = Some(
            z_session
                .declare_subscriber(key_gear_command)
                .callback_mut(move |sample| {
                    match cdr::deserialize_from::<_, autoware_auto_vehicle_msgs::GearCommand, _>(
                        &*sample.payload.contiguous(),
                        cdr::size::Infinite,
                    ) {
                        Ok(gearcmd) => {
                            //println!("GearCommand: {}\r", gearcmd.command);
                            gear_cmd.store(gearcmd.command, Ordering::Relaxed);
                        }
                        Err(_) => {}
                    }
                })
                .res()
                .unwrap(),
        );
        let current_velocity = self.current_velocity.clone();
        self._subscriber_velocity = Some(
            z_session
                .declare_subscriber(key_velocity)
                .callback_mut(move |sample| {
                    match cdr::deserialize_from::<_, autoware_auto_vehicle_msgs::VelocityReport, _>(
                        &*sample.payload.contiguous(),
                        cdr::size::Infinite,
                    ) {
                        Ok(velocity) => {
                            //println!("Velocity: {}\r", velocity.longitudinal_velocity);
                            current_velocity
                                .store(velocity.longitudinal_velocity, Ordering::Relaxed);
                        }
                        Err(_) => {}
                    }
                })
                .res()
                .unwrap(),
        );

        let steering_tire_angle = self.steering_tire_angle.clone();
        let target_velocity = self.target_velocity.clone();
        let gear_cmd = self.gear_command.clone();
        let current_velocity = self.current_velocity.clone();
        let publisher_control_command = z_session
            .declare_publisher(key_control_command)
            .res()
            .unwrap();
        thread::spawn(move || {
            loop {
                //println!("v:{} angle:{}\r", target_velocity.load(Ordering::Relaxed),
                //                            steering_tire_angle.load(Ordering::Relaxed));
                let real_target_velocity = target_velocity.load(Ordering::Relaxed)
                    * (if gear_cmd.load(Ordering::Relaxed) == 2 {
                        1.0
                    } else {
                        -1.0
                    });
                let acceleration = num::clamp(
                    target_velocity.load(Ordering::Relaxed)
                        - current_velocity.load(Ordering::Relaxed).abs(),
                    -1.0,
                    1.0,
                );
                // TODO: This should be filled with current time
                let empty_time = builtin_interfaces::Time { sec: 0, nanosec: 0 };
                let control_cmd = autoware_auto_control_msgs::AckermannControlCommand {
                    stamp: empty_time.clone(),
                    lateral: autoware_auto_control_msgs::AckermannLateralCommand {
                        stamp: empty_time.clone(),
                        steering_tire_angle: steering_tire_angle.load(Ordering::Relaxed),
                        steering_tire_rotation_rate: 0.0,
                    },
                    longitudinal: autoware_auto_control_msgs::LongitudinalCommand {
                        stamp: empty_time.clone(),
                        speed: real_target_velocity,
                        acceleration,
                        jerk: 0.0,
                    },
                };
                let encoded = cdr::serialize::<_, _, CdrLe>(&control_cmd, Infinite).unwrap();
                publisher_control_command.put(encoded).res().unwrap();
                thread::sleep(Duration::from_millis(33)); // 30 Hz
            }
        });
    }

    fn pub_gate_mode(&self, mode: u8) {
        let gate_mode_data = tier4_control_msgs::GateMode { data: mode };
        let encoded = cdr::serialize::<_, _, CdrLe>(&gate_mode_data, Infinite).unwrap();
        self.publisher_gate_mode.put(encoded).res().unwrap();
    }

    fn send_client_engage(&self) {
        // TODO: We assign GUID and seq to 0, but this should be filled with meaningful value.
        let engage_data = autoware_auto_vehicle_msgs::EngageRequest {
            header: service::ServiceHeader { guid: 0, seq: 0 },
            mode: true,
        };
        let encoded = cdr::serialize::<_, _, CdrLe>(&engage_data, Infinite).unwrap();
        self.client_engage_req.put(encoded).res().unwrap();
    }

    pub fn toggle_gate_mode(&self) -> bool {
        // Return whether switch to external or not
        if self.gate_mode.load(Ordering::Relaxed) == 0 {
            // Auto
            self.pub_gate_mode(1);
            self.send_client_engage();
            true
        } else {
            // External
            self.pub_gate_mode(0);
            false
        }
    }

    pub fn pub_gear_command(&self, command: u8) {
        let gear_command = autoware_auto_vehicle_msgs::GearCommand {
            stamp: builtin_interfaces::Time { sec: 0, nanosec: 0 },
            command: command,
        };
        let encoded = cdr::serialize::<_, _, CdrLe>(&gear_command, Infinite).unwrap();
        self.publisher_gear_command.put(encoded).res().unwrap();
    }

    pub fn update_control_command(&self, velocity: f32, angle: f32) {
        self.steering_tire_angle.store(angle, Ordering::Relaxed);
        self.target_velocity.store(velocity, Ordering::Relaxed);
    }

    pub fn get_status(&self) -> String {
        let mut s = String::from("Enage:");
        s += if self.current_engage.load(Ordering::Relaxed) {
            "Ready"
        } else {
            "Not Ready"
        };
        s += "\tGate Mode:";
        s += match self.gate_mode.load(Ordering::Relaxed) {
            0 => "Auto",
            1 => "External",
            _ => "Unknown",
        };
        s += "\tGear:";
        // TODO: Use const for gear command
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
