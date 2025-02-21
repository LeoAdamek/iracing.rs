use iracing::telemetry::Connection;

pub fn main() {
    let conn = Connection::new().expect("Unable to open telemetry");
    let session = conn.session_info().expect("Invalid session data");

    print!("{:#?}", session);
}
