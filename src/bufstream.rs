use std::io::{BufRead, BufReader, BufWriter, Read, Write};

struct InternalBufWriter<W>
where
    W: Write,
{
    writer: BufWriter<W>,
}

impl<W> Read for InternalBufWriter<W>
where
    W: Read + Write,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.writer.get_mut().read(buf)
    }
}

impl<W> Write for InternalBufWriter<W>
where
    W: Read + Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

pub struct BufStream<W>
where
    W: Write,
{
    stream: BufReader<InternalBufWriter<W>>,
}

impl<S> BufStream<S>
where
    S: Read + Write,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream: BufReader::new(InternalBufWriter {
                writer: BufWriter::new(stream),
            }),
        }
    }
}

impl<S> Read for BufStream<S>
where
    S: Read + Write,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<S> BufRead for BufStream<S>
where
    S: Read + Write,
{
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.stream.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.stream.consume(amt)
    }
}

impl<S> Write for BufStream<S>
where
    S: Read + Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.get_mut().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.get_mut().flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::VecDeque,
        io::{Read, Write},
    };

    #[test]
    fn test_run() {
        let buff: VecDeque<u8> = VecDeque::new();
        let mut stream = BufStream::new(buff);
        stream.write_all(&[1, 2, 3, 4]).unwrap();
        stream.flush().unwrap();
        let mut buff = Vec::new();
        stream.read_to_end(&mut buff).unwrap();
        assert_eq!(vec![1, 2, 3, 4], buff);
    }
}
