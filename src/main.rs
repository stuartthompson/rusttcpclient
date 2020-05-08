// extern crate tui;

use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::{self, Receiver, TryRecvError};
// use tui::Terminal;
// use tui::backend::CrosstermBackend;

fn main() {
    if std::env::args().len() != 3 {
        println!("Usage: rusttcpclient ip port");
        return;
    }

    // Parse command-line arguments
    let ip = std::env::args()
        .nth(1)
        .expect("Expected argument 1 to be IP.");
    let port = std::env::args()
        .nth(2)
        .expect("Expected argument 2 to be port.");

    // Print out startup information
    println!("TCP Client");
    println!("~~~~~~~~~~");
    println!();
    println!("IP: {0}", ip);
    println!("Port: {0}", port);

    let addr = format!("{0}:{1}", ip, port);

    match TcpStream::connect(addr) {
        Ok(stream) => {
            println!("[Client] Connected to server at {0}:{1}", ip, port);

            stream
                .set_nonblocking(true)
                .expect("Failed to set stream to non-blocking.");

            client_loop(stream);
        }
        Err(error) => {
            println!("Failed to connect to server. Error: {0}", error);
        }
    }
    println!("[Client] Quitting");
}

fn client_loop(mut stream: std::net::TcpStream) {
    // Establish a channel to listen to stdin
    let stdin_rx = listen_to_stdin();

    // Echo input commands
    print_commands();

    // Receive buffer (for message received back from server)
    let mut data = [0 as u8; 512]; // using 512 byte buffer

    loop {
        // Check for input on stdin
        match stdin_rx.try_recv() {
            Ok(input) => { 
                let cmd: &str = &input.trim();
                match cmd {
                    "Q" => {
                        println!("Quitting!");
                        break;
                    }
                    "H" => {
                        println!("Say hello");
                        let msg = b"Hello!";
                        stream.write(msg).expect("Error writing to TCP stream");
                    }
                    "D" => {
                        println!("Disconnecting...");
                        stream
                            .shutdown(Shutdown::Both)
                            .expect("Error shutting down stream.");
                    }
                    "S" => {
                        println!("Send custom string");
                        let mut custom = String::new();
                        std::io::stdin()
                            .read_line(&mut custom)
                            .expect("Error reading custom string to send.");
                        stream
                            .write(custom.as_bytes())
                            .expect("Error writing custom stream to TCP stream.");
                    }
                    &_ => {
                        println!("Command not recognized. Received: {0}", cmd);
                    }
                }
            },
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }

        match stream.read(&mut data) {
            Ok(size) => {
                let msg = std::str::from_utf8(&data[0..size]).unwrap();
                println!("Received {0} bytes: {1}", size, msg);
            }
            // Handle case where waiting for accept would become blocking
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            }
            Err(error) => {
                println!("Error reading from stream: {0}", error);
            }
        }

        // Sleep for 10ms
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn listen_to_stdin() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        loop {
            let mut io_stream = std::io::stdin(); // Get io stream
            let mut stdin_buf = [0 as u8; 16]; // 16 byte buffer
            match io_stream.read(&mut stdin_buf) {
                Ok(0) => {}
                Ok(size) => {
                    let stdin_input = std::str::from_utf8(&stdin_buf[0..size]).unwrap();
                    tx.send(String::from(stdin_input))
                        .expect("Error sending on channel.");
                }
                Err(e) => {
                    println!("Error reading from stdin stream: {0}", e);
                }
            }
        }
    });

    // Return receiver so others may listen
    return rx;
}

fn print_commands() {
    println!("[Q] to quit");
    println!("[H] to send 'Hello'");
    println!("[D] to disconnect");
    println!("[S] to send custom string");
}
