use iracing::Connection;
use serde_yaml::to_string;

pub fn main() {
    let conn = Connection::new().expect("Unable to open telemetry");
    let telem = conn.telemetry().expect("Telem");

    print!("{}", to_string(&telem.all()).unwrap());
}