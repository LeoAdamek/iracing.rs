# `0.5.0`:

## Breaking Changes

All telemetry features are now inside the `telemetry` mod, and will only be compiled if the target OS is `windows` and the `telemetry` feature is enabled.
All references to `iracing::Connection` will need to be changed to `iracing::telemetry::Connection` and the `telemetry` feature added to your Cargo.toml:

e.g. `iracing = {version = 0.5, features = ["telemetry"] }