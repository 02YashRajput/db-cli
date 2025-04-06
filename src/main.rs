use regex::Regex;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let address = if args.len() == 1 {
        
        "localhost:4000".to_string()
    } else if args.len() == 2 && args[1].starts_with("--") {
        let addr = &args[1][2..];
        if addr.contains(':') {
            addr.to_string()
        } else {
            eprintln!("Invalid address format. Use --<ip:port>");
            std::process::exit(1);
        }
    } else {
        eprintln!("Usage:");
        eprintln!("  db               => connects to localhost:4000");
        eprintln!("  db --<ip:port>   => connects to given IP and port");
        std::process::exit(1);
    };

    let stream = TcpStream::connect(&address).await?;
    println!("Connected to {}", address);
    println!("Entering REPL mode. Type 'exit' to quit.");

    let (read_half, mut write_half) = stream.into_split();
    let mut server_reader = BufReader::new(read_half);
    let mut server_line = String::new();
    let stdin = io::stdin();
    let mut prompt = String::from("db> ");

    loop {
        print!("{}", prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let trimmed = input.trim();

        if trimmed.eq_ignore_ascii_case("exit") {
            println!("Bye!");
            break;
        }

        write_half.write_all(input.as_bytes()).await?;

        loop {
            server_line.clear();
            let bytes_read = server_reader.read_line(&mut server_line).await?;

            if bytes_read == 0 {
                println!("Server closed the connection.");
                return Ok(());
            }

            let response = server_line.trim();

            // Username and Password prompts
            if response.eq_ignore_ascii_case("Enter username:") || response.eq_ignore_ascii_case("Username:") {
                print!("{} ", response);
                io::stdout().flush()?;
                let mut username = String::new();
                stdin.read_line(&mut username)?;
                write_half.write_all(username.as_bytes()).await?;
                continue;
            }

            if response.eq_ignore_ascii_case("Enter password:") || response.eq_ignore_ascii_case("Password:") {
                print!("{} ", response);
                io::stdout().flush()?;
                let password = rpassword::read_password().unwrap_or_default();
                write_half.write_all((password + "\n").as_bytes()).await?;
                continue;
            }

            // yes/no prompt
            if response == "Do you want authentication (yes/no)?" {
                print!("{} ", response);
                io::stdout().flush()?;
                let mut answer = String::new();
                stdin.read_line(&mut answer)?;
                write_half.write_all(answer.as_bytes()).await?;
                continue;
            }

            println!("{}", response);

            // Update prompt on database selection
            if response.contains("Using database") {
                if let Some(dbname) = Regex::new(r"'(.*?)'")
                    .unwrap()
                    .captures(response)
                    .and_then(|cap| cap.get(1))
                {
                    prompt = format!("{}> ", dbname.as_str());
                }
            }

            break;
        }
    }

    Ok(())
}
