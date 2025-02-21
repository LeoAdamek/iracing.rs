use iracing::states::Flags;
use iracing::telemetry::Connection;
use std::convert::TryInto;
use std::time::Duration;

pub fn main() {
    let conn = Connection::new().expect("Unable to open telemetry");
    let session = conn.session_info().expect("Invalid Session");

    let shift_up_rpm = session.drivers.shift_light_shift_rpm;
    let blocking = conn.blocking().expect("Couldn't create telem handle");

    loop {
        let telem = match blocking.sample(Duration::from_millis(500)) {
            Ok(sample) => sample,
            Err(err) => {
                println!("Error: {:?}", err);
                continue;
            }
        };

        let rpm: f32 = telem.get("RPM").unwrap().try_into().unwrap();
        let gear: i32 = telem.get("Gear").unwrap().try_into().unwrap();
        let timecode: f64 = telem.get("SessionTime").unwrap().try_into().unwrap();
        let lap: i32 = telem.get("Lap").unwrap().try_into().unwrap();

        let flags =
            Flags::from_bits(telem.get("SessionState").unwrap().try_into().unwrap()).unwrap();

        print!(
            "Lap {lap:>3}: {time:>5.3}s Gear {gear} @ {rpm:>5.0} RPM ",
            lap = lap,
            gear = gear,
            rpm = rpm,
            time = timecode
        );

        if shift_up_rpm < rpm {
            print!("{:>8}", " SHIFT ");
        }

        println!("{:#?}({:})", flags, flags.bits());
    }
}
