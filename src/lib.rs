#[macro_use]
extern crate bitflags;

extern crate memmap;

use memmap::*;

pub mod session;
pub mod telemetry;

pub const TELEMETRY_PATH: &str = "Local\\IRSDKMemMapFileName";
pub const UNLIMITED_LAPS: i32 = 32767;
pub const UNLIMITED_TIME: f32 = 604800.0;

pub fn open() -> Result<Client, String> {

}

pub struct Client { }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
