use std::convert::TryInto;
use iracing::Connection;
use iracing::telemetry::Value;
use iracing::telemetry::EngineWarnings;
use std::thread::sleep;
use std::time::Duration;

pub fn main() {    
    let mut conn = Connection::new().expect("Unable to open telemetry");

    let session = conn.session_info().expect("Invalid Session");
    let shift_up_rpm = session.drivers.shift_light_shift_rpm;

    let blocking = unsafe { conn.blocking() }.expect("Couldn't create telem handle");

    loop {
        let telem = match blocking.sample(Duration::from_millis(500)) {
            Ok(sample) => sample,
            Err(err) => {
                println!("Error: {:?}", err);
                continue;
            }
        };

        let rpm: f32 = telem.get("RPM").unwrap().into();
        let gear: i32 = telem.get("Gear").unwrap().into();
        let timecode: f64 = telem.get("SessionTime").unwrap().into();
        let lap: i32 = telem.get("Lap").unwrap().into();

        print!("Lap {lap:>3}: {time:>5.3}s Gear {gear} @ {rpm:>5.0} RPM", lap = lap, gear = gear, rpm = rpm, time = timecode);
        
        if shift_up_rpm < rpm {
            print!("  SHIFT");
        }

        println!("");

        // sleep(Duration::from_millis(25));
    }

}