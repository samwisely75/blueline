# blueline

A lightweight, profile-based HTTP client with REPL interface.

## Overview

Blueline is a modern command-line HTTP client featuring:

- **REPL Interface**: Interactive Read-Eval-Print Loop with vi-style navigation
- **Profile-based Configuration**: Store and reuse connection settings, headers, and authentication
- **Security-first Design**: REPL-only mode prevents piped input vulnerabilities
- **Split-pane UI**: Request editing and response viewing in separate panes
- **JSON Support**: Automatic JSON formatting and syntax highlighting
- **Verbose Mode**: Detailed connection and request/response information

## Installation

### From Source

```bash
git clone https://github.com/samwisely75/blueline
cd blueline
cargo build --release
```

### From Package Managers

> **Note**: Package manager releases coming soon

## Quick Start

### Basic Usage

```bash
# Start blueline with default profile
blueline

# Start with specific profile
blueline -p staging

# Start with verbose output
blueline -v
```

### Profile Configuration

Create `~/.blueline/profile` with your API configurations:

```ini
[default]
host = https://api.example.com
@content-type = application/json
@accept = application/json

[staging]
host = https://staging-api.example.com
@authorization = Bearer your-token-here
user = username
password = secret

[local]
host = http://localhost:8080
@content-type = application/json
insecure = true
```

### REPL Commands

Once in the REPL interface:

- **Navigation**: Use vi-style keys (`h`, `j`, `k`, `l`) to move around
- **Insert Mode**: Press `i` to enter insert mode for editing
- **Normal Mode**: Press `Esc` to return to normal mode
- **Execute Request**: Press `Enter` to send the HTTP request
- **Quit**: Press `:q` or `Ctrl+C` to exit

## Command Line Options

```
blueline [OPTIONS]

Options:
  -p, --profile <PROFILE>    Use specified profile from ~/.blueline/profile [default: default]
  -v, --verbose              Enable verbose output showing connection details
  -h, --help                 Print help information
  -V, --version              Print version information
```

## Profile Configuration Reference

### Connection Settings

- `host` - Base URL for API endpoints (required)
- `insecure` - Skip TLS certificate verification (`true`/`false`)
- `ca_cert` - Path to custom CA certificate file
- `proxy` - HTTP/HTTPS proxy URL

### Authentication

- `user` - Username for Basic Authentication
- `password` - Password for Basic Authentication

### Headers

Prefix header names with `@`:

- `@authorization` - Authorization header
- `@content-type` - Content-Type header
- `@accept` - Accept header
- `@user-agent` - User-Agent header

### Example Profile

```ini
[production]
host = https://api.production.com
@authorization = Bearer prod-token-here
@content-type = application/json
@accept = application/json
user = api-user
password = secure-password
ca_cert = /etc/ssl/certs/production-ca.pem
proxy = http://corporate-proxy:8080
insecure = false
```

## REPL Interface

### Split-Pane Layout

```
┌─────────────────────────────────┐
│          Request Pane           │
│                                 │
│ Method: GET                     │
│ Path: /api/users                │
│ Body: {"name": "John"}          │
│                                 │
├─────────────────────────────────┤
│         Response Pane           │
│                                 │
│ Status: 200 OK                  │
│ {                               │
│   "users": [...]                │
│ }                               │
└─────────────────────────────────┘
```

### Vi-Style Navigation

- `h` - Move cursor left
- `j` - Move cursor down  
- `k` - Move cursor up
- `l` - Move cursor right
- `w` - Move to next word
- `b` - Move to previous word
- `0` - Move to beginning of line
- `$` - Move to end of line

### Editing Commands

- `i` - Enter insert mode at cursor
- `a` - Enter insert mode after cursor
- `o` - Insert new line below and enter insert mode
- `x` - Delete character under cursor
- `dd` - Delete current line
- `u` - Undo last change

## Verbose Mode

When using `-v` flag, blueline displays detailed information:

### Connection Details

```text
> connection:
>   host: https://api.example.com
>   port: 443
>   scheme: https
>   ca-cert: <none>
>   insecure: false
>   headers:
>    content-type: application/json
>    authorization: Bearer token
>   proxy: <none>
```

### Request Information

```text
> request:
>   method: POST
>   path: /api/users
>   body: {"name": "John", "email": "john@example.com"}
```

### Response Details

```text
> response:
>   status: 201 Created
>   headers:
>     content-type: application/json
>     location: /api/users/123
```

## Security Features

- **No piped input**: Prevents injection attacks through stdin
- **Profile isolation**: Credentials stored separately from commands
- **TLS verification**: Certificate validation enabled by default
- **Secure defaults**: Conservative configuration options

## Architecture

Blueline uses the [bluenote](https://github.com/samwisely75/bluenote) HTTP client library for profile-based HTTP requests. The bluenote library provides reusable HTTP client functionality with configuration management that can be used independently in other Rust projects.

## Development

### Building

```bash
# Build entire workspace
cargo build

# Build specific crate (now just blueline)
cargo build --package blueline

# Run tests
cargo test
```

### Project Structure

```text
blueline/
├── blueline/           # REPL application crate
│   ├── src/
│   │   ├── main.rs     # Application entry point
│   │   ├── cmd.rs      # Command line parsing
│   │   └── repl.rs     # REPL interface
│   └── tests/          # Integration tests
└── Cargo.toml          # Workspace configuration
```

## License

Licensed under the Elastic License 2.0. See LICENSE file for details.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. **Install Git hooks**: Run `./scripts/install-hooks.sh` to set up pre-commit clippy checks
4. Make your changes
5. Add tests for new functionality
6. Ensure all tests pass (hooks will enforce clippy compliance)
7. Submit a pull request

### Development Setup

The project includes pre-commit hooks to maintain code quality:

```bash
# Install Git hooks (required for contributors)
./scripts/install-hooks.sh
```

The pre-commit hook will:

- Run `cargo clippy --all-targets --all-features -- -D warnings`
- Reject commits with any clippy warnings
- Ensure modern format string syntax (e.g., `format!("Hello {name}")`)

To bypass in emergencies: `git commit --no-verify`

## Related Projects

- **httpc**: Command-line HTTP client (predecessor)
- **curl**: Classic HTTP client tool
- **httpie**: Modern HTTP client
- **postman**: GUI API testing tool
