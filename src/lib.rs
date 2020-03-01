#[macro_use]
extern crate bitflags;

extern crate memmap;

use memmap::Mmap;
use std::fs::OpenOptions;

pub mod session;
pub mod telemetry;

pub const TELEMETRY_PATH: &str = "Local\\IRSDKMemMapFileName";
pub const UNLIMITED_LAPS: i32 = 32767;
pub const UNLIMITED_TIME: f32 = 604800.0;

pub fn connect() -> std::io::Result<Client> {
    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(TELEMETRY_PATH)?;

    let _ = unsafe {
        Mmap::map(&file)?;
    };

    return Ok(Client::new());
}

pub struct Client { 
}

impl Client {
    pub fn new() -> Client {
        Client { }
    }
}

#[cfg(test)]
mod tests {
    use super::connect;

    #[test]
    fn test_connect() {
        connect().expect("Unable to do the telemetry");
        assert_eq!(1,1);
    }
}
