#![allow(dead_code)] // We may not need to use functions we export
#[macro_use]
extern crate bitflags;
extern crate serde;
extern crate serde_yaml;
extern crate encoding;
extern crate chrono;

#[cfg(all(target_os="windows", feature="telemetry"))]
extern crate winapi;
#[cfg(all(target_os="windows", feature="telemetry"))]
extern crate libc;

pub mod session;
pub mod track_surface;
pub mod states;
pub mod replay;

#[cfg(all(target_os="windows",feature="telemetry"))]
pub mod telemetry;