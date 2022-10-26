use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::{bufstream::BufStream, value::ValueRead};

trait Client<T>
where
    T: Read + Write,
{
    fn handle(&mut self, connection: &mut T);
}

impl<F, T> Client<T> for F
where
    T: Read + Write,
    F: Fn(&mut T),
{
    fn handle(&mut self, connection: &mut T) {
        (self)(connection)
    }
}

fn handle_connection(connection: &mut TcpStream) {
    let mut stream = BufStream::new(connection);
    loop {
        let value = stream.read_value();
        match value {
            Ok(v) => todo!(),
            Err(err) => todo!(),
        }
    }
}
