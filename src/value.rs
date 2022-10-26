use std::{
    borrow::Borrow,
    io,
    num::ParseIntError,
    ops::{Deref, DerefMut},
    string::FromUtf8Error,
};

#[derive(Debug, Eq, PartialEq)]
pub enum Value {
    Simple(Bytes),
    Blob(Bytes),
    Number(i64),
    Array(Vec<Value>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Bytes(Vec<u8>);

impl<S> From<&S> for Bytes
where
    S: Borrow<str>,
{
    fn from(s: &S) -> Self {
        Self(s.borrow().bytes().into_iter().collect())
    }
}

impl Deref for Bytes {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    ParseError,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_: FromUtf8Error) -> Self {
        Self::ParseError
    }
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Self::ParseError
    }
}

pub trait ValueWrite: io::Write {
    fn write_value(&mut self, value: &Value) -> Result<(), Error> {
        match value {
            Value::Simple(buff) => {
                self.write("+".as_bytes())?;
                self.write(buff.as_slice())?;
                self.write("\r\n".as_bytes())?;
            }
            Value::Blob(buff) => {
                self.write("$".as_bytes())?;
                self.write(buff.as_slice())?;
                self.write("\r\n".as_bytes())?;
            }
            Value::Number(num) => {
                self.write(":".as_bytes())?;
                self.write(format!("{}", num).as_bytes())?;
                self.write("\r\n".as_bytes())?;
            }
            Value::Array(slice) => {
                self.write("*".as_bytes())?;
                self.write(format!("{}", slice.len()).as_bytes())?;
                for elem in slice.iter() {
                    self.write_value(elem)?;
                }
            }
        }
        Ok(())
    }
}

impl<W: io::Write + ?Sized> ValueWrite for W {}

trait ValueReadExt: io::BufRead {
    fn read_number(&mut self) -> Result<i64, Error> {
        let mut buff = Vec::new();
        self.read_until('\n' as u8, &mut buff)?;
        buff.pop();
        buff.pop();

        Ok(String::from_utf8(buff)?.parse()?)
    }
}

impl<R: io::BufRead + ?Sized> ValueReadExt for R {}

pub trait ValueRead: io::BufRead {
    fn read_value(self: &mut Self) -> Result<Value, Error> {
        let mut buff: [u8; 1] = [0];
        self.read_exact(&mut buff)?;

        let value = match buff[0] as char {
            '+' => {
                let mut buff = Vec::new();
                self.read_until('\n' as u8, &mut buff)?;
                buff.pop();
                buff.pop();
                Value::Simple(Bytes(buff))
            }
            '$' => {
                let num = self.read_number()?;
                let mut buff = vec![0u8; num as usize];
                self.read_exact(&mut buff)?;
                self.consume(2);
                Value::Blob(Bytes(buff))
            }
            ':' => {
                let num = self.read_number()?;
                Value::Number(num)
            }
            '*' => {
                let num = self.read_number()?;
                let mut values = vec![];
                for _ in 0..num {
                    values.push(self.read_value()?);
                }
                Value::Array(values)
            }
            _ => todo!(),
        };

        Ok(value)
    }
}

impl<R: io::BufRead + ?Sized> ValueRead for R {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let mut buffer: Vec<u8> = vec![];
        let val = Value::Simple(Bytes(vec![1]));
        buffer.write_value(&val).unwrap();
    }
}
