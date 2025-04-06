// =======================================================
// 🧠 INFO: Imports
// =======================================================

use regex::Regex; // For extracting database name from response
use std::io::{self, Write}; // Standard IO
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader}; // Async buffered IO
use tokio::net::TcpStream; // Async TCP connection

// =======================================================
// 🚀 Main Async Entry Point
// =======================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ---------------------------------------------------
    // 📥 INFO: Process CLI Arguments to get address
    // ---------------------------------------------------
    let args: Vec<String> = std::env::args().collect();

    // Default to localhost or parse from --<ip:port>
    let address = if args.len() == 1 {
        "localhost:4000".to_string()
    } else if args.len() == 2 && args[1].starts_with("--") {
        let addr = &args[1][2..];
        if addr.contains(':') {
            addr.to_string()
        } else {
            // ⚠️ WARNING: Invalid format
            eprintln!("Invalid address format. Use --<ip:port>");
            std::process::exit(1);
        }
    } else {
        // ⚠️ USAGE Help
        eprintln!("Usage:");
        eprintln!("  db               => connects to localhost:4000");
        eprintln!("  db --<ip:port>   => connects to given IP and port");
        std::process::exit(1);
    };

    // ---------------------------------------------------
    // 🔌 INFO: Connect to the Server
    // ---------------------------------------------------
    let stream = TcpStream::connect(&address).await?;
    println!("Connected to {}", address);
    println!("Entering REPL mode. Type 'exit' to quit.");

    // Split stream into readable & writable parts
    let (read_half, mut write_half) = stream.into_split();

    // Wrap reading in a buffered reader
    let mut server_reader = BufReader::new(read_half);
    let mut server_line = String::new();
    let stdin = io::stdin();

    // Default prompt
    let mut prompt = String::from("db> ");

    // =======================================================
    // 🔁 REPL Loop: Read-Eval-Print Loop
    // =======================================================
    loop {
        // 👉 Display prompt
        print!("{}", prompt);
        io::stdout().flush()?; // Flush to actually show it

        let mut input = String::new();
        stdin.read_line(&mut input)?; // Read user input
        let trimmed = input.trim();

        // ❌ Exit Condition
        if trimmed.eq_ignore_ascii_case("exit") {
            println!("Bye!");
            break;
        }

        // ✉️ Send user input to the server
        write_half.write_all(input.as_bytes()).await?;

        // 🔁 Handle multi-line server response
        loop {
            server_line.clear();
            let bytes_read = server_reader.read_line(&mut server_line).await?;

            // ❌ Server closed the connection
            if bytes_read == 0 {
                println!("Server closed the connection.");
                return Ok(());
            }

            let response = server_line.trim();

            // 🔐 Handle Username Prompt
            if response.eq_ignore_ascii_case("Enter username:")
                || response.eq_ignore_ascii_case("Username:")
            {
                print!("{} ", response);
                io::stdout().flush()?;
                let mut username = String::new();
                stdin.read_line(&mut username)?;
                write_half.write_all(username.as_bytes()).await?;
                continue;
            }

            // 🔐 Handle Password Prompt
            if response.eq_ignore_ascii_case("Enter password:")
                || response.eq_ignore_ascii_case("Password:")
            {
                print!("{} ", response);
                io::stdout().flush()?;
                // 🔒 Secure password input (no echo)
                let password = rpassword::read_password().unwrap_or_default();
                write_half.write_all((password + "\n").as_bytes()).await?;
                continue;
            }

            // 🔐 Handle yes/no authentication prompt
            if response == "Do you want authentication (yes/no)?" {
                print!("{} ", response);
                io::stdout().flush()?;
                let mut answer = String::new();
                stdin.read_line(&mut answer)?;
                write_half.write_all(answer.as_bytes()).await?;
                continue;
            }

            // 📤 Print server response
            println!("{}", response);

            // 🧠 NOTE: Update prompt if server confirms DB change
            if response.contains("Using database") {
                if let Some(dbname) = Regex::new(r"'(.*?)'")
                    .unwrap()
                    .captures(response)
                    .and_then(|cap| cap.get(1))
                {
                    prompt = format!("{}> ", dbname.as_str());
                }
            }

            // ✅ Done processing this response
            break;
        }
    }

    Ok(())
}
