# Database Client (REPL)

A simple TCP-based database client with REPL (Read-Eval-Print Loop) interface that connects to a database server.

## Features

- Connects to a database server via TCP
- Supports authentication (username/password)
- REPL interface for interactive commands
- Dynamic prompt changes when selecting databases
- Handles various server prompts automatically

## Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- For password input: `rpassword` crate is used (included in dependencies)
- - Compatible with [db-server](https://github.com/02YashRajput/db-server)

## Installation

1. Clone the repository 
2. Run using cargo run 

## Usage
### Basic Connection
Connect to the default server (localhost:4000):

```bash
cargo run 
```

### Custom Server Connection
Connect to a specific server using --<ip:port> syntax:

```bash
cargo run --192.168.1.100:5000
```
run 
```bash
cargo run --20.169.221.119:4000 to get access to the online db 
```



### REPL Commands
Once connected:

+ Type database commands to execute them

+ Type exit to quit the session

The prompt will change when you select a database (e.g., from db> to mydb> ).

### Authentication
If the server requires authentication:

+ You'll be prompted for username and password

+  Password input is hidden for security

### Building a Standalone Binary
To create a standalone binary:

```bash
cargo build --release
```
The binary will be available at:
+ Linux/macOS: target/release/db

+ Windows: target/release/db.exe

You can copy this binary to any location in your PATH for easy access.

### Dependencies
+ tokio - For async I/O and networking

+ regex - For parsing database names from server responses

+ rpassword - For secure password input

+ anyhow - For error handling

