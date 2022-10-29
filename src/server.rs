use crate::bufstream::BufStream;
use crate::config::Config;
use crate::db::db::{Database, Session};
use crate::error::Error;
use crate::value::{ValueRead, ValueWrite};
use log;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::thread;

pub struct Server<'a> {
    addr: String,
    database: &'a mut Database,
}

impl<'a> Server<'a> {
    pub fn new(config: &Config, database: &'a mut Database) -> Self {
        Self {
            addr: format!("{}:{}", config.host, config.port),
            database,
        }
    }

    pub fn run(&self) -> io::Result<()> {
        thread::scope(|server_scope| {
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

                let session = self.database.create_session();
                server_scope.spawn(move || {
                    Self::handle_connection(session, connection);
                });
            }

            Ok(())
        })
    }

    fn handle_connection(mut session: Session, connection: TcpStream) {
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

            let response = session.handle_request(val);

            match stream.write_value(&response) {
                Ok(_) => (),
                Err(err) => {
                    log::error!(
                        "Error writing response to client{}: {}. Disconnecting",
                        addr,
                        err
                    );
                    break;
                }
            }
        }
    }
}
