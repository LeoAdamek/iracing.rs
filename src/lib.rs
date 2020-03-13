#![allow(dead_code)] // We may not need to use functions we export
#[macro_use]
extern crate bitflags;
extern crate winapi;
extern crate serde;
extern crate serde_yaml;
extern crate encoding;
extern crate libc;

use crate::telemetry::Header;
use crate::session::*;
use serde_yaml::from_str as yaml_from;
use std::ptr::null;
use std::io::Error;
use std::slice::from_raw_parts;
use std::io::Result as IOResult;
use std::mem::transmute;
use std::os::windows::raw::HANDLE;
use std::sync::Mutex;
use libc::c_void;
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_1;
use winapi::shared::minwindef::LPVOID;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::memoryapi::{OpenFileMappingW, FILE_MAP_READ, MapViewOfFile};

pub mod session;
pub mod telemetry;
pub mod track_surface;
pub mod states;

pub const TELEMETRY_PATH: &'static str = "Local\\IRSDKMemMapFileName";
pub const UNLIMITED_LAPS: i32 = 32767;
pub const UNLIMITED_TIME: f32 = 604800.0;

///
/// iRacing live telemetry and session data connection.
/// 
/// Allows retrival of live data fro iRacing.
/// The data is provided using a shared memory map, allowing the simulator
/// to deposit data roughly every 16ms to be read.
/// 
/// # Examples
/// 
/// ```
/// use iracing::Connection;
/// 
/// let _ = Connection::new().expect("Unable to find telemetry data");
/// ```
pub struct Connection {
    mux: Mutex<()>,
    location: *const c_void,
    header: Header
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

        let header = unsafe { Self::read_header(view) };

        return Ok(Connection {mux: Mutex::default(), location: view, header: header});
    }

    ///
    /// Get the data header
    /// 
    /// Reads the data header from the shared memory map and returns a copy of the header
    /// which can be used safely elsewhere.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use iracing::Connection;
    /// 
    /// let location_of_an_iracing_header: *const c_void;
    /// let header = Connection::read_header(location_of_an_iracing_header);
    /// 
    /// println!("Data Version: {}", header.version);
    /// ```
    pub unsafe fn read_header(from: *const c_void) -> Header {
        let raw_header: *const Header = transmute(from);
        let h: Header = *raw_header;

        h.clone()
    }

    ///
    /// Get session information
    /// 
    /// Get general session information - This data is mostly static and contains
    /// overall information related to the current or replayed session
    /// 
    /// # Examples
    /// 
    /// ```
    /// use iracing::Connection;
    /// 
    /// match Connection::new().expect("Unable to open session").session_info() {
    ///     Ok(session) => println!("Track Name: {}", session.weekend.track_display_name),
    ///     Err(e) => println!("Invalid Session")
    /// };
    /// ```
    pub fn session_info(&mut self) -> Result<SessionDetails, Box<dyn std::error::Error>> {
        let header = unsafe { Self::read_header(self.location) };

        let start = (self.location as usize + header.session_info_offset as usize) as *const u8;
        let size = header.session_info_length as usize;
        
        let data: &[u8] = unsafe { from_raw_parts(start, size) };

        // Decode the data as ISO-8859-1 (Rust wants UTF-8)
        let content: String = match ISO_8859_1.decode(data, DecoderTrap::Strict) {
            Ok(value) => value,
            Err(e) => return Err(Box::from(e))
        };

        match yaml_from(content.as_str()) {
            Ok(session) => Ok(session),
            Err(e) => Err(Box::from(e))
        }
    }

    ///
    /// Get latest telemetry.
    /// 
    /// Get the latest live telemetry data, the telemetry is updated roughtly every 16ms
    /// 
    /// # Examples
    /// 
    /// ```
    /// use iracing::Connection;
    /// 
    /// let sample = Connection::new()?.telemetry()?;
    /// ``` 
    pub fn telemetry(&self) -> Result<telemetry::Sample, Box<dyn std::error::Error>> {
        let header = unsafe { Self::read_header(self.location) };
        header.telemetry(self.location as *const std::ffi::c_void)
    }

    ///
    /// Get Blocking Telemetry Interface.
    /// 
    /// Creates a new `iracing::telemetry::Blocking` connector which allows telemetry samples to
    /// be collected, and will wait and block until a new sample is available, or a timeout is reached.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use iracing::Connection;
    /// use std::time::Duration;
    /// 
    /// let sampler = Connection::new()?.blocking()?;
    /// let sample = sample.sample(Duration::from_millis(50))?;
    /// ```
    pub fn blocking(&self) -> IOResult<telemetry::Blocking> {
        telemetry::Blocking::new(self.location, unsafe { Self::read_header(self.location) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_info() {
        match Connection::new().expect("Unable to open telemetry").session_info() {
            Ok(session) => println!("Track: {}", session.weekend.track_name),
            Err(e) => println!("Error: {:?}", e)
        };
    }

    #[test]
    fn test_latest_telemetry() {
        Connection::new().expect("Unable to open telemetry").telemetry().expect("Couldn't get latest telem");
    }
}
