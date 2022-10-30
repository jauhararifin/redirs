use std::{
    borrow::Borrow,
    fmt::Display,
    io,
    ops::{Deref, DerefMut},
};

use crate::error::Result;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Value {
    Simple(Bytes),
    Blob(Bytes),
    Number(i64),
    Array(Vec<Value>),
    Err(String, String),
    Null,
}

impl Value {
    pub fn err(message: impl Into<String>) -> Self {
        Self::Err("ERR".to_string(), message.into())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO(jauhararifin): write proper implementation for this.
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Bytes(Vec<u8>);

impl Bytes {
    pub fn into_string(self) -> Result<String> {
        Ok(String::from_utf8(self.0)?)
    }
}

impl<S> From<&S> for Bytes
where
    S: Borrow<str> + ?Sized,
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

pub trait ValueWrite: io::Write {
    fn write_value(&mut self, value: &Value) -> Result<()> {
        match value {
            Value::Simple(buff) => {
                self.write("+".as_bytes())?;
                self.write(buff.as_slice())?;
                self.write("\r\n".as_bytes())?;
            }
            Value::Blob(buff) => {
                self.write("$".as_bytes())?;
                self.write(format!("{}", buff.len()).as_bytes())?;
                self.write("\r\n".as_bytes())?;
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
                self.write("\r\n".as_bytes())?;
                for elem in slice.iter() {
                    self.write_value(elem)?;
                }
            }
            Value::Err(code, msg) => {
                self.write("-".as_bytes())?;
                self.write(code.as_bytes())?;
                self.write(" ".as_bytes())?;
                self.write(msg.as_bytes())?;
                self.write("\r\n".as_bytes())?;
            }
            Value::Null => {
                self.write("$-1\r\n".as_bytes())?;
            }
        }
        self.flush()?;
        Ok(())
    }
}

impl<W: io::Write + ?Sized> ValueWrite for W {}

trait ValueReadExt: io::BufRead {
    fn read_number(&mut self) -> Result<i64> {
        let mut buff = Vec::new();
        self.read_until('\n' as u8, &mut buff)?;
        buff.pop();
        buff.pop();

        Ok(String::from_utf8(buff)?.parse()?)
    }
}

impl<R: io::BufRead + ?Sized> ValueReadExt for R {}

pub trait ValueRead: io::BufRead {
    fn read_value(self: &mut Self) -> Result<Value> {
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
                if num < 0 {
                    Value::Null
                } else {
                    let mut buff = vec![0u8; num as usize];
                    self.read_exact(&mut buff)?;
                    self.consume(2);
                    Value::Blob(Bytes(buff))
                }
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
        let testcases = vec![
            (
                Value::Simple("somesimplestring".into()),
                "+somesimplestring\r\n",
            ),
            (
                Value::Blob("somesimplestring".into()),
                "$16\r\nsomesimplestring\r\n",
            ),
            (Value::Number(-1), ":-1\r\n"),
            (Value::Number(0), ":0\r\n"),
            (Value::Number(12912), ":12912\r\n"),
            (
                Value::Array(vec![
                    Value::Simple("loremipsum".into()),
                    Value::Blob("doscolorsit".into()),
                    Value::Number(123),
                ]),
                "*3\r\n+loremipsum\r\n$11\r\ndoscolorsit\r\n:123\r\n",
            ),
        ];

        for (value, expected) in testcases {
            let mut buffer: Vec<u8> = vec![];
            buffer.write_value(&value).unwrap();
            assert_eq!(expected, String::from_utf8(buffer).unwrap().as_str());
        }
    }

    #[test]
    fn test_deserialization() {
        let testcases = vec![
            (
                "+somesimplestring\r\n",
                Value::Simple("somesimplestring".into()),
            ),
            (
                "$16\r\nsomesimplestring\r\n",
                Value::Blob("somesimplestring".into()),
            ),
            (":-1\r\n", Value::Number(-1)),
            (":0\r\n", Value::Number(0)),
            (":12912\r\n", Value::Number(12912)),
            (
                "*3\r\n+loremipsum\r\n$11\r\ndoscolorsit\r\n:123\r\n",
                Value::Array(vec![
                    Value::Simple("loremipsum".into()),
                    Value::Blob("doscolorsit".into()),
                    Value::Number(123),
                ]),
            ),
        ];

        for (input, expected) in testcases {
            let val = input.as_bytes().read_value().unwrap();
            assert_eq!(expected, val);
        }
    }
}
