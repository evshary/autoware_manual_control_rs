use zenoh::prelude::sync::*;
use zenoh::subscriber::Subscriber;
use zenoh::publication::Publisher;
use zenoh::buffers::reader::HasReader;
use serde_derive::{Serialize, Deserialize};
use cdr::{CdrLe, Infinite};

pub struct ManualController<'a> {
    pub_gate_mode: Publisher<'a>,
    _sub_gate_mode: Subscriber<'a, ()>,
}

impl<'a> ManualController<'a> {
    pub fn new(z_session: &'a Session) -> Self {
        let pub_gate_mode = z_session
            .declare_publisher("rt/control/gate_mode_cmd")
            .res()
            .unwrap();

        let sub_gate_mode = z_session
            .declare_subscriber("rt/control/current_gate_mode")
            .callback_mut(move |sample| {
                match cdr::deserialize_from::<_, GateMode, _>(sample.payload.reader(), cdr::size::Infinite) {
                    Ok(gatemode) => {
                        println!("gatemode.date={}\r", gatemode.data);
                    }
                    Err(_) => {}
                }
            })
            .res()
            .unwrap();

        ManualController {
            pub_gate_mode,
            _sub_gate_mode: sub_gate_mode
        }
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
