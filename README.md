# Würth Elektronik Sensor Communication

Rust drivers for communicating with the full range of sensors that Würth Elektronik produce.

## Cargo Workspace

This repo uses a cargo workspace to manage all of the crates inside. One neat feature we're using is the abilities to define common dependencies like `embedded-hal` once at the top level and include them in the crates. This helps us ensure we're using compatible dependency versions across each crate.
