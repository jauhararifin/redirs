use std::io;
use std::net::TcpListener;
use std::thread;

use crate::bufstream::BufStream;
use crate::value::ValueRead;

pub struct Config {
    pub host: String,
    pub port: i16,
}

pub struct Server<'a> {
    config: &'a Config,
}

impl<'a> Server<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let server = TcpListener::bind(format!("{}:{}", self.config.host, self.config.port))?;
        for client in server.incoming() {
            let connection = match client {
                Ok(conn) => conn,
                Err(_) => todo!(),
            };

            thread::spawn(move || {
                let mut stream = BufStream::new(connection);
                loop {
                    let val = stream.read_value();
                    let val = match val {
                        Ok(val) => val,
                        Err(_) => break,
                    };
                    println!("Got value {:?}", val);
                }
            });
        }

        Ok(())
    }
}
