# KRPCGEN

`krpcgen` is a cli tool inspired by
[`rpcgen`](https://docs.oracle.com/cd/E19683-01/816-1435/6m7rrfn7f/index.html).

The main difference is that `krpcgen` targets linux kernel and creates
pair of loadable modules for client and server sides.

## Installation

Right now only build from source option is available.

```bash
git clone https://github.com/muji40k/krpcgen.git
cd krpcgen
cargo install --path .
```

## Usage

```
Usage: krpcgen [OPTIONS]

Options:
  -p, --path <PATH>                    Path to workspace [default: .]
  -s, --specification <SPECIFICATION>  Path to rpcl specificaion file [default: spec.x]
  -v, --vla-limit <VLA_LIMIT>          Constant value for maximum variable lenght array size [default: 1024]
  -h, --help                           Print help
  -V, --version                        Print version
```

## Module structure

Before loading modules make sure to load `sunrpc` module.

```bash
sudo modprobe sunrpc
```

```
.
├── clients                     ── client modules stored here
│   ├── <program_name>          ── directory per module
│   │   ├── <version_name>      ── version definition
│   │   │   ├── constants.h     ── procedure numbers
│   │   │   ├── procedure_api.c ── exported procedures, that can be used
│   │   │   ├── procedure_api.h    from other modules
│   │   │   ├── procedures.h    ── misc
│   │   │   ├── procedure_xdr.c ── encoding and decoding
│   │   │   ├── version.c
│   │   │   └── version.h
│   │  !├── authentication.c    ── user authentication logic
│   │   ├── authentication.h
│   │   ├── constants.h         ── version numbers
│   │   └── program.c           ── program definition and module entrypoint
│   ├── client.c                ── common client handling
│   └── client.h
├── servers                     ── server modules stored here
│   ├── <program_name>          ── derictory per module
│   │   ├── <version_name>      ── version definition
│   │   │   ├── constants.h     ── procedure numbers
│   │   │  !├── procedure_handlers.c ── actual procedure handlers
│   │   │   ├── procedures.h    ── procedure definitions
│   │   │   ├── procedure_xdr.c ── encoding/decoding/release
│   │   │   ├── version.c
│   │   │   └── version.h
│   │  !├── authentication.c    ── user authentication logic
│   │   ├── authentication.h
│   │   ├── constants.h         ── version numbers
│   │   └── program.c           ── program definition and module entrypoint
│   ├── common.c                ── common functions (threadfn and dispatch)
│   └── common.h
├── constants.h                 ── defined constants and enums
├── types.h                     ── other types
└── Makefile
```

Files marked with `!` contains actual application logic and must be updated by
user. Feel free to mess around with other files too.

Exported remote procedure calls are provided with `procedure_api.h` and can be
called from other loadable modules.

## Configuration

Generated modules provide some kind of configuration:

### Server

`port` - port for server to operate on (short) \[default: 0\]

`threads` - number of handler threads (int) \[default: 1\]

### Client

`version` - which version client should use (int) \[default: 1\]

`host_ip` - destination ip (char[4]) \[default: 127,0,0,1\]

`port` - destination port (short) \[default: 0 (uses `rpcbind`)\]

