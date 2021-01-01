# `0.5.0`:

## ⚠ Breaking Changes

All telemetry features are now inside the `telemetry` mod, and will only be compiled if the target OS is `windows` and the `telemetry` feature is enabled.
All references to `iracing::Connection` will need to be changed to `iracing::telemetry::Connection` and the `telemetry` feature added to your Cargo.toml:

e.g. `iracing = {version = 0.5, features = ["telemetry"] }


## �� New Features

Replays , allows reading of replay data to acquire certain metadata from replays. Replays can come from anything implementing the `std::io::Read` trait, such as files or network streams.

Find it as `iracing::replay`

```rust
use std::fs::File;
use std::iracing::Replay;
let src = File::open("replay.rpy").expect("Unable to open replay file");
let replay = Replay::new(src).expect("Invalid replay file");
```