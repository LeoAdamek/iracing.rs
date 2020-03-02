#[macro_use]
extern crate bitflags;

extern crate winapi;

use crate::session::Header;
use std::ptr::null;
use std::io::Error;
use std::io::Result as IOResult;
use std::os::windows::raw::HANDLE;
use winapi::shared::minwindef::LPVOID;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::memoryapi::{OpenFileMappingW, FILE_MAP_READ, MapViewOfFile};

pub mod session;
pub mod telemetry;

pub const TELEMETRY_PATH: &'static str = "Local\\IRSDKMemMapFileName";
pub const UNLIMITED_LAPS: i32 = 32767;
pub const UNLIMITED_TIME: f32 = 604800.0;


pub struct Client {
    conn: Connection,
}

impl Client {
    pub fn new(conn: Connection) -> Client {
        return Client { conn: conn };
    }

    pub fn header(&mut self) -> Header {
        self.conn.header()
    }
}

pub struct Connection {
    location: *mut std::ffi::c_void
}

impl Connection {
    pub fn new() -> IOResult<Connection> {
        let mut path: Vec<u16> = TELEMETRY_PATH.encode_utf16().collect();
        path.push(0);

        let mapping: HANDLE;
        let errno: i32;

        unsafe { mapping = OpenFileMappingW(FILE_MAP_READ, 0, path.as_ptr()); };

        if null() == mapping {
            
            unsafe { errno = GetLastError() as i32; }

            return Err(Error::from_raw_os_error(errno));
        }

        let view: LPVOID;
 
        unsafe {
            view = MapViewOfFile(mapping, FILE_MAP_READ, 0, 0, 0);
        }

        if null() == view {
            unsafe { errno = GetLastError() as i32; }

            return Err(Error::from_raw_os_error(errno))
        }

        return Ok(Connection {location: view});
    }

    /**
     * Read the contents of the memory map
     */
    pub fn header (&mut self) -> Header {
        let raw_header: *const Header = unsafe { std::mem::transmute(self.location) };
        let h: Header = unsafe { *raw_header };

        h.clone()
    }
}

pub fn connect() -> IOResult<Client> {
    let conn = Connection::new()?;

    return Ok(Client::new(conn));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect() {
        println!("Opening telemetry at {}", TELEMETRY_PATH);
        connect().expect("Unable to open telemetry");
        assert_eq!(1, 1);
    }

    #[test]
    fn test_header() {
        println!("Sampling Telemetry");

        let header: Header = connect().expect("Unable to open telemetry").header();

        assert_eq!(header.version, 2);
    }
}
