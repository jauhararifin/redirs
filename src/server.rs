use crate::bufstream::BufStream;
use crate::config::Config;
use crate::value::ValueRead;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn run(config: &Config) -> io::Result<()> {
    let addr = format!("{}:{}", config.host, config.port);
    let server = TcpListener::bind(addr)?;
    for client in server.incoming() {
        let connection = match client {
            Ok(conn) => conn,
            Err(_) => todo!(),
        };

        thread::spawn(move || {
            handle_connection(connection);
        });
    }

    Ok(())
}

fn handle_connection(connection: TcpStream) {
    let mut stream = BufStream::new(connection);
    loop {
        let val = stream.read_value();
        let val = match val {
            Ok(val) => val,
            Err(_) => break,
        };
        println!("Got value {:?}", val);
    }
}
