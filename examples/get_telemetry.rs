use iracing::Connection;
use iracing::telemetry::Value;
use std::thread::sleep;
use std::time::Duration;

pub fn main() {    
    let mut conn = Connection::new().expect("Unable to open telemetry");

    let session = conn.session_info().expect("Invalid Session");
    let shift_up_rpm = session.drivers.shift_light_shift_rpm;

    loop {
        let telem = conn.telemetry().expect("No telemetry");

        let rpm = telem.get("RPM");
        let gear = telem.get("Gear");

        match gear {
            Some(Value::INT(g)) => {
                print!("{}  ", g);
            }
            _ => {}
        }

        match rpm {
            Some(Value::FLOAT(rpms)) => {
                print!("RPM = {}", rpms);

                if rpms >= shift_up_rpm {
                    print!(" SHIFT");
                }

                println!("",);
            }

            _ => {}
        }

        // println!("RPM = {:?}", telem.get("RPM"));
        sleep(Duration::from_millis(20));
    }

}