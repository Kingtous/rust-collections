use std::{sync::mpsc::channel, thread};

use rdev::{simulate, EventType};

fn main() {
    let (tx, rx) = channel();
    let mouse_event = EventType::MouseMove { x: 200.0, y: 200.0 };
    let msg = serde_json::to_string(&mouse_event).unwrap();
    let _ = tx.send(msg);
    let t = thread::spawn(move || {
        if let Ok(s) = rx.recv() {
            let evt: EventType = serde_json::from_str(&s).unwrap();
            let _ = simulate(&evt);
            println!("recv event {:?}", evt);
        }
    });
    t.join().unwrap();
}
