use std::io::{self, ErrorKind, Read, Write};

use mio::{Interest, Registry, Token, net::TcpStream};

const MAX_LINE_LEN: usize = 4096;
const MAX_WRITE_BUFFER: usize = 1024 * 1024;

pub(crate) enum Queued {
    NeedsFlush,
    AlreadyPending,
    Overflow,
    Closing,
}

pub(crate) enum ReadStatus {
    Drained,
    MoreToRead,
}

pub(crate) struct Connection {
    pub token: Token,
    stream: TcpStream,
    ip: String,
    write_buffer: Vec<u8>,
    read_buffer: Vec<u8>,
    closing: bool,
}

impl Connection {
    pub(crate) fn new(stream: TcpStream, token: Token, ip: String) -> Self {
        Self {
            stream,
            token,
            ip,
            read_buffer: Vec::with_capacity(MAX_LINE_LEN),
            write_buffer: Vec::with_capacity(4096),
            closing: false,
        }
    }
    pub(crate) fn token(&self) -> Token {
        self.token
    }
    pub(crate) fn ip(&self) -> &str {
        &self.ip
    }
    pub(crate) fn has_buffered_output(&self) -> bool {
        !self.write_buffer.is_empty()
    }
    pub(crate) fn register(&mut self, registry: &Registry, interest: Interest) -> io::Result<()> {
        registry.register(&mut self.stream, self.token, interest)
    }
    pub(crate) fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        registry.deregister(&mut self.stream)
    }
    pub(crate) fn drain_outgoing_messages(&mut self) -> io::Result<()> {
        while !self.write_buffer.is_empty() {
            match self.stream.write(&self.write_buffer) {
                Ok(0) => return Err(ErrorKind::WriteZero.into()),
                Ok(n) => {
                    self.write_buffer.drain(..n);
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub(crate) fn read_lines(
        &mut self,
        budget: usize,
        mut on_line: impl FnMut(&mut Self, &str),
    ) -> io::Result<ReadStatus> {
        let mut tmp = [0u8; 4096];
        for _ in 0..budget {
            let n = match self.stream.read(&mut tmp) {
                Ok(0) => return Err(ErrorKind::UnexpectedEof.into()),
                Ok(n) => n,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(ReadStatus::Drained),
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            self.read_buffer.extend_from_slice(&tmp[..n]);
            self.for_each_line(&mut on_line);
            if self.read_buffer.len() > MAX_LINE_LEN {
                return Err(io::Error::new(ErrorKind::InvalidData, "line too long"));
            }
        }
        Ok(ReadStatus::MoreToRead)
    }

    fn for_each_line(&mut self, mut f: impl FnMut(&mut Self, &str)) {
        let mut buf = std::mem::take(&mut self.read_buffer);
        let mut start = 0;
        while let Some(pos) = buf[start..].iter().position(|&b| b == b'\n') {
            let line = String::from_utf8_lossy(&buf[start..start + pos]);
            f(self, &line);
            start += pos + 1;
        }
        buf.drain(..start);
        self.read_buffer = buf;
    }

    pub(crate) fn queue_message(&mut self, message: &str) -> Queued {
        if self.closing {
            return Queued::Closing;
        }
        if self.write_buffer.len() + message.len() + 1 > MAX_WRITE_BUFFER {
            self.closing = true;
            return Queued::Overflow;
        }
        let was_empty = self.write_buffer.is_empty();
        self.write_buffer.extend_from_slice(message.as_bytes());
        self.write_buffer.push(b'\n');
        if was_empty {
            Queued::NeedsFlush
        } else {
            Queued::AlreadyPending
        }
    }
}
