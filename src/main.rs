// -------------------- IMPORTS --------------------

// Importing necessary modules from the standard library
use std::io::{self, Write}; // For synchronous input/output and flushing stdout (used for CLI)

// Importing async I/O traits and utilities from Tokio
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader}; // For async buffered reading and writing
use tokio::net::TcpStream; // For establishing a TCP connection

// Importing the Url parser crate to easily extract parts of a URL
use url::Url;

// -------------------- MAIN FUNCTION --------------------

// The entry point of the async Tokio runtime
#[tokio::main] // This macro sets up the Tokio runtime so we can use `.await` in main
async fn main() -> anyhow::Result<()> {
    // Collect command-line arguments into a vector of strings
    let args: Vec<String> = std::env::args().collect();

    // Ensure exactly one argument is passed (the connection URI)
    if args.len() != 2 {
        eprintln!("Usage: db db://<username>:<password>@<host>:<port>");
        std::process::exit(1); // Exit the program with error code 1
    }

    let uri = &args[1]; // Get the URI string from the second argument

    // Parse the URI using the `url` crate
    let parsed_url = Url::parse(uri)?;
    let username = parsed_url.username(); // Extract the username
    let password = parsed_url.password().unwrap_or(""); // Extract password or use empty string
    let host = parsed_url.host_str().unwrap_or("localhost"); // Extract host or use "localhost"
    let port = parsed_url.port().unwrap_or(8080); // Extract port or use default 8080

    // Format the address string like "localhost:8080"
    let address = format!("{}:{}", host, port);

    // Attempt to connect to the server at the given address (TCP server)
    let mut stream = TcpStream::connect(&address).await?;
    println!("Connected to {}", address);

    // -------------------- AUTHENTICATION --------------------

    // Format and send the AUTH command to the server (✉️ To TCP server)
    let auth_command = format!("AUTH {} {}\n", username, password);
    stream.write_all(auth_command.as_bytes()).await?; // Send the command to TCP server

    // Wrap the stream in a buffered reader for efficient reading (📥 From TCP server)
    let mut reader = BufReader::new(stream);
    let mut response = String::new(); // String to hold server response
    reader.read_line(&mut response).await?; // Read one line from the TCP server

    // If the response is not "AUTH OK", then fail authentication
    if response.trim() != "AUTH OK" {
        eprintln!("Authentication failed: {}", response.trim());
        return Ok(()); // Exit gracefully
    }

    println!("Authentication successful. Entering REPL mode. Type 'exit' to quit.");

    // -------------------- REPL MODE --------------------

    // Split the reader into separate read/write halves (to interact independently)
    let (read_half, mut write_half) = reader.into_inner().into_split();

    // Re-wrap the read half in a BufReader for reading from the TCP server
    let mut server_reader = BufReader::new(read_half); // 📥 TCP server responses

    let mut server_line = String::new(); // To store server responses
    let stdin = io::stdin(); // 📥 CLI input (blocking synchronous read from user)

    // Start the interactive command loop (REPL)
    loop {
        print!("db> "); // Display prompt (🖨️ To CLI)
        io::stdout().flush()?; // Flush output so prompt appears immediately

        let mut input = String::new(); // Buffer for user input
        stdin.read_line(&mut input)?; // 📥 Read input from CLI

        let trimmed = input.trim(); // Remove extra whitespace

        // Exit condition
        if trimmed.eq_ignore_ascii_case("exit") {
            println!("Bye!"); // 🖨️ Output to CLI
            break;
        }

        // Send user input to the TCP server (✉️ To TCP server)
        write_half.write_all(input.as_bytes()).await?;

        // Read the server's response (📥 From TCP server)
        server_line.clear(); // Clear previous contents
        server_reader.read_line(&mut server_line).await?;

        // Print the server's reply (🖨️ To CLI)
        println!("{}", server_line.trim());
    }

    Ok(()) // Return success
}
