use zenoh::prelude::sync::*;
use zenoh::subscriber::Subscriber;
use zenoh::publication::Publisher;
use zenoh::buffers::reader::HasReader;
use serde_derive::{Serialize, Deserialize};
use cdr::{CdrLe, Infinite};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

pub struct ManualController<'a> {
    pub_gate_mode: Publisher<'a>,
    _sub_gate_mode: Option<Subscriber<'a, ()>>,
    
    gate_mode: Arc<AtomicU8>,
}

impl<'a> ManualController<'a> {
    pub fn new(z_session: &'a Session) -> Self {
        let pub_gate_mode = z_session
            .declare_publisher("rt/control/gate_mode_cmd")
            .res()
            .unwrap();

        ManualController {
            pub_gate_mode,
            _sub_gate_mode: None,
            gate_mode: Arc::new(AtomicU8::new(0)),
        }
    }

    pub fn init(&mut self, z_session: &'a Session) {
        let gate_mode = self.gate_mode.clone();
        self._sub_gate_mode = Some(z_session
            .declare_subscriber("rt/control/current_gate_mode")
            .callback_mut(move |sample| {
                match cdr::deserialize_from::<_, GateMode, _>(sample.payload.reader(), cdr::size::Infinite) {
                    Ok(gatemode) => {
                        //println!("gatemode.date={}\r", gatemode.data);
                        gate_mode.store(gatemode.data, Ordering::Relaxed);
                    }
                    Err(_) => {}
                }
            })
            .res()
            .unwrap());
    }

    pub fn pub_gate_mode(&self, mode: u8) {
        let gate_mode_data = GateMode { data: mode};
        let encoded = cdr::serialize::<_, _, CdrLe>(&gate_mode_data, Infinite).unwrap();
        self.pub_gate_mode.put(encoded).res().unwrap();
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
struct GateMode {
    data: u8,  // 0: AUTO, 1: EXTERNAL
}
