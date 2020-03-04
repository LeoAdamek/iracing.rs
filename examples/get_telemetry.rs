

use iracing::Connection;
use std::thread::sleep;
use std::time::Duration;

pub fn main() {    
    let mut conn = Connection::new().expect("Unable to open telemetry");

    loop {
        let telem = conn.get_telemetry().expect("No telemetry");
        println!("RPM = {:?}", telem.get("RPM"));
        sleep(Duration::from_millis(20));
    }

}