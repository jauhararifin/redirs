use crate::bufstream::BufStream;
use crate::config::Config;
use crate::error::Error;
use crate::value::{Value, ValueRead};
use log;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::thread;

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(config: &Config) -> Self {
        Self {
            addr: format!("{}:{}", config.host, config.port),
        }
    }

    pub fn run(&self) -> io::Result<()> {
        log::info!("Starting server at {}", &self.addr);

        let server = TcpListener::bind(&self.addr)?;
        for client in server.incoming() {
            let connection = match client {
                Ok(conn) => conn,
                Err(err) => {
                    log::error!("Cannot accept connection: {:?}", err);
                    continue;
                }
            };

            thread::spawn(move || {
                Self::handle_connection(connection);
            });
        }

        Ok(())
    }

    fn handle_connection(connection: TcpStream) {
        let addr = match connection.local_addr() {
            Ok(addr) => addr.to_string(),
            Err(err) => {
                log::error!("Cannot get client address: {}", err);
                "unknown_address".to_string()
            }
        };

        log::info!("Client connected: {}", addr);

        let mut stream = BufStream::new(connection);
        loop {
            let val = stream.read_value();
            let val = match val {
                Ok(val) => val,
                Err(Error::Eof) => {
                    log::info!("Client disconnected: {}", addr);
                    break;
                }
                Err(err) => {
                    log::error!(
                        "Error reading command from client {}: {}. Disconnecting",
                        addr,
                        err
                    );
                    break;
                }
            };

            Self::handle_command(val);
        }
    }

    fn handle_command(value: Value) {
        println!("Got command: {:?}", value);
    }
}
