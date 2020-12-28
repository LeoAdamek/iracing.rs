iRacing.rs
==========

Live telemetry and session data interface for Rust.

Features are available on all platforms by default, except for live telemetry which is available only on Windows and requires the `telemetry` feature to be enabled.

Usage
-----

See further examples in [/examples](examples/)

```rust
extern crate iracing;
use iracing::telemetry::{Connection, TelemetryError};

pub fn main() {

    // Open the iRacing Telemetry data
    let conn = Connection::new().expect("Unable to open telemetry. Is iRacing running?");

    // Get a blocking telemetry client
    let bc = Connection::blocking().expect("Unable to start telemetry reader");

    loop {
        // bc.get() will block until new telemetry data is available.
        let sample = match bc.get() {
            Ok(sample) => sample,
            Err(TelemetryError::TIMEOUT(ms)) => panic!("Telemetry timed out after {}ms", ms);
            Err(error) => panic!("Telemetry Error: {:?}", error);
        }
    }
}
```


How iRacing Telemetry Works
---------------------------

iRacing provides very little documentation on how the telemetry data is exported, 
so here's my version of it.

iRacing exports telemetry to a [non-persisted memory-mapped file](https://docs.microsoft.com/en-us/dotnet/standard/io/memory-mapped-files).  
This allows iRacing to provide telemetry data with a high update rate which
can be read by multiple reading applications.

The shared memory space is always called `Local\\IRSDKMemMapFileName`.  
This memory contains four main areas:

* A top-level header which describes the content of the memory space including:
  * The data version (currently 2.0)
  * The update rate (usually 60Hz)
  * The game-tick when the data was last updated
  * Information needed to find and read the session information and telemetry.

* An ISO-8859-1 encoded YAML string containing semi-static session information
  such as the name and layout of the track, the cars being driven and the
  users driving those cars.

* A secondary header which describes the data available in the telemetry buffers
* Up to 4 telemetry data buffers

The simulator cycles through up to 4 telemetry data buffers when writing telemetry
and updates the top-level header to indicate when each buffer was last updated
and where it is located. All buffers share the same structure, the number of values
available is fixed per-session.

The session data can be read as a string given the location and size indicated
by the top-level header and parsed as YAML to get the full details of the
session. The structure of the YAML document is provided in the IRSDK documentation.

The telemetry data available is variable and depends primarily on the player's car.
The top-level header denotes how many telemetry values are available and a pointer
to the start of an array of structures which describe these contents.
The structure is as follows:

```
typedef struct iracing_telem_var_header {
    int value_type /* Enum of value type */
    int offset /* Offset from start of telemetry buffer where variable is stored */
    int count /* A count of values for this variable */

    char [3]pad /* Padding */

    char [32]name /* Varaible name */
    char [64]desc /* Variable description */
    char [32]units /* Variable units */
}
```

If the top-level header indicates there are 548 variables,
then the the variables header will be an array of 548 items (`iracing_telem_var_header[548]`).
This header can then used as a look-up-table to find specific telemetry variables
within the telemetry buffer.

For example, given the following variable header:

```c
{
    .value_type = 1, /* float */
    .offset = 0x4F82,
    .count = 6,
    .pad = [0,0,0],
    .name = "DampDeflectLR",
    .desc = "Damper Deflection (Left-Rear)",
    .units = "mm"
};
```

We know that the variable "DampDeflectLR" exists `0x4F82` bytes from the start
of the telemetry buffer, the values are floats, of 4-bytes each and there are 6
values.

Knowing this we will need to read 24 bytes starting `0x4F82` bytes from the start
of the telemetry buffer to `0x4F93` which will give us an array of 6 `float`s

A C implementation would look like this:
```c
float* suspension_deflect = (float*)calloc(6, sizeof(float));
size_t suspection_deflect_loc = 0x4F82;

memcpy(suspension_deflect, telem_buffer_start + suspension_deflect_loc, 6 * sizeof(float));
```