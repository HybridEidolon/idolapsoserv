# IDOLA PSO Server Emulator

A Phantasy Star Online emulator written in Rust. Currently very incomplete.

[![Build Status](https://travis-ci.org/BygoneWorlds/idolapsoserv.svg?branch=master)](https://travis-ci.org/BygoneWorlds/idolapsoserv)

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

## Running

The `idola` binary is multi-functional. Primarily, it is the server, but it
is multiple servers combined. The configuration file passed with the `-c` or
`--config` option specifies what services are run in this instance of the
server. This aids in debugging; an example configuration can be found at
`data/default/idola_local.toml` in the source tree that spawns a patch, login,
ship, 10 blocks, and the shipgate server all bound on 127.0.0.1, with the
shipgate's database configured to a Sqlite file named `local.db` in the current
directory.

A complete Blue Burst service minimally requires at least one of each:

* BB Patch server
* BB Data server
* BB Login server (configured to redirect to itself)
* Ship
* Block
* Shipgate

The shipgate serves information about accounts and all the other services
(except for the patch server) must login and verify on it. Ships will talk to
the shipgate to register themselves to the server.

_The following section is not implemented yet._

In addition to running the services, `idola` can be used to execute commands
on a shipgate, such as account management.

### Shipgate Security concerns

Currently, only a simple password is used to authenticate connections to the
shipgate. That means, if you are running an Internet-accessible service, you
should configure your network such that your services connect through your own
intranet, so that shipgate traffic does not route through the Internet. If not,
user credentials and the shipgate password could be exposed. The easiest way to
ensure this for a single server is to bind the shipgate on localhost and
configure services to connect on localhost as well.

## License

Copyright (C) 2015, 2016 Bygone Worlds Project

Portions derived from Sylverant, Copyright (C) 2012, 2013, 2014, 2015 Lawrence Sebald

> This program is free software: you can redistribute it and/or modify
> it under the terms of the GNU Affero General Public License as published by
> the Free Software Foundation, either version 3 of the License, or
> (at your option) any later version.
>
> This program is distributed in the hope that it will be useful,
> but WITHOUT ANY WARRANTY; without even the implied warranty of
> MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
> GNU General Public License for more details.
>
> You should have received a copy of the GNU Affero General Public License
> along with this program.  If not, see <http://www.gnu.org/licenses/>.
