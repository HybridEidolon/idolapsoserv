# IDOLA PSO Server Emulator

A Phantasy Star Online emulator written in Rust. Currently very incomplete.

## Features

* Native Windows, Linux, and MacOSX support.
* Written in Rust, a high level systems language with statically guaranteed memory and concurrency safety, and a smart optimizing compiler.
* Memory-efficient architecture for handling connections.
* Supports Blue Burst.

### Planned features

* Support for multiple kinds of databases (currently sqlite3 file store only)
* Flexible configuration system for single or multi server set-ups. Only want to run the game locally? Just configure IDOLA to spin up every service needed by the game locally, and run a single instance of the app.
* Experience point and drop rate adjustment in config for PSOBB.
* Cross-version interaction between all versions of PSO.

## Building

IDOLA requires Rust 1.5 stable or later to build.

1. Install Rust 1.5 for your platform. Check with `rustc --version`
2. Install `libsqlite3`.
3. Run `cargo build`. Optionally, `cargo build --release` for release optimizations. Warning: compile times will skyrocket.
4. The binary will be in `target/{debug|release}/idola[.exe]`.

### Windows

**You must use the GNU ABI version of Rust** to build because we link into `libsqlite3`. The easiest way to set this up is to install MSYS2, and install the `mingw-w64-i686` package for `libsqlite3` (_not_ the MSYS version) in Pacman. The build script for the Rust binding will use the Windows version of `pkg-config` to find the libsqlite3 to link.

## License

This will probably be licensed under AGPLv3. Portions of the project are slightly derivative of [sylverant](https://github.com/Sylverant), and that's under AGPLv3, so it is prudent of this to use it.
