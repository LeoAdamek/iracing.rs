

use iracing::Connection;

pub fn main() {    
    let mut conn = Connection::new().expect("Unable to open telemetry");
    let telem = conn.get_telemetry().expect("No telemetry");

    println!("{:#?}", telem);

}