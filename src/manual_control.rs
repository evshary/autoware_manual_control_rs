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
    client_engage_req: Publisher<'a>,
    _sub_gate_mode: Option<Subscriber<'a, ()>>,
    _sub_engage: Option<Subscriber<'a, ()>>,
    
    gate_mode: Arc<AtomicU8>,
}

impl<'a> ManualController<'a> {
    pub fn new(z_session: &'a Session) -> Self {
        let pub_gate_mode = z_session
            .declare_publisher("rt/control/gate_mode_cmd")
            .res()
            .unwrap();
        let client_engage_req = z_session
            .declare_publisher("rq/api/autoware/set/engageRequest")
            .res()
            .unwrap();

        ManualController {
            pub_gate_mode,
            client_engage_req,
            _sub_gate_mode: None,
            _sub_engage: None,
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
        self._sub_engage = Some(z_session
            .declare_subscriber("rt/api/autoware/get/engage")
            .callback_mut(move |sample| {
                match cdr::deserialize_from::<_, GetEngage, _>(sample.payload.reader(), cdr::size::Infinite) {
                    Ok(_engage) => {
                        //println!("Engage: {} {}\r", engage.ts.sec, engage.enable);
                    }
                    Err(_) => {}
                }
            })
            .res()
            .unwrap());
    }

    fn pub_gate_mode(&self, mode: u8) {
        let gate_mode_data = GateMode { data: mode };
        let encoded = cdr::serialize::<_, _, CdrLe>(&gate_mode_data, Infinite).unwrap();
        self.pub_gate_mode.put(encoded).res().unwrap();
    }

    fn send_client_engage(&self) {
        let engage_data = Engage { header: ServiceHeader { guid: 0, seq: 0 }, enable: true };
        let encoded = cdr::serialize::<_, _, CdrLe>(&engage_data, Infinite).unwrap();
        self.client_engage_req.put(encoded).res().unwrap();
    }

    pub fn toggle_gate_mode(&self) {
        if self.gate_mode.load(Ordering::Relaxed) == 0 { // Auto
            self.pub_gate_mode(1);
            self.send_client_engage();
        } else { // External
            self.pub_gate_mode(0);
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
struct GateMode {
    data: u8,  // 0: AUTO, 1: EXTERNAL
}

#[derive(Serialize, Deserialize, PartialEq)]
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
struct Engage {
    header: ServiceHeader,
    enable: bool,
}

/*
#[derive(Serialize, Deserialize, PartialEq)]
struct ResponseStatus {
    header: ServiceHeader,
    code: u32,
    message: String,
}
*/

#[derive(Serialize, Deserialize, PartialEq)]
struct GetEngage {
    ts: TimeStamp,
    enable: bool,
}
