//! Fast, naive implementation of Redis protocol.

#[cfg(test)]
mod tests;

/// Additional functionalities for types that implements [`std::io::Write`].
///
/// [`std::io::Write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html
pub mod write {
  /// Provides set of functionalities to make communicating easier.
  pub trait WriteExt {
    /// Writes line ending in CRLF.
    ///
    /// If you're looking for more general function, see `write_line_as`.
    fn write_line(&mut self, data: impl AsRef<[u8]>) -> std::io::Result<()>;
  }

  impl<T: std::io::Write> WriteExt for std::io::BufWriter<T> {
    fn write_line(&mut self, data: impl AsRef<[u8]>) -> std::io::Result<()> {
      use std::io::Write;
      return self
        .write_all(data.as_ref())
        .and_then(|_| self.write_all(b"\r\n"))
        .and_then(|_| self.flush());
    }
  }
}

/// Error type for Redis library.
pub mod error {
  #[derive(Debug)]
  /// Represents error for dealing with Redis TCP protocol.
  pub enum Error {
    /// Signifies that reading from stream returned empty data.
    ReadEmptyData,
    /// Signifies that reading failed.
    FailedDataRead,
    /// Means that Redis returned some unknown type.
    ///
    /// This can mean possible corruption during TCP transfer or change in the protocol.
    UnknownType,
  }
  impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
      use Error::*;
      return write!(fmt, "{}", match self {
        ReadEmptyData => "Read empty data",
        FailedDataRead => "Failed to read data",
        UnknownType => "Unknown return type",
      })
    }
  }
}

/// Return type of redis execution, as specified in Redis protocol [specification].
///
/// [specification]: https://redis.io/topics/protocol
#[derive(Debug, PartialEq)]
pub enum ReturnType {
  /// Null type, returned when there is no meaningful data.
  Null,
  /// Represents single binary safe string.
  BulkString {
    data: String,
    size: usize
  },
  /// Represents an integer, is string according to the specification.
  Integer(String),
  /// Represents a return status, either success (`OK`) or failure (`ERR`).
  State(String),
  /// Represents an array of arbitrary values.
  Array(Vec<Box<ReturnType>>),
}
/// Represents connection between Redis server and this client.
pub struct Connection {
  reader: std::io::BufReader<std::net::TcpStream>,
  writer: std::io::BufWriter<std::net::TcpStream>,
}
impl Connection {
  /// Create new instance of `Connection`.
  ///
  /// Connects with Redis server through TCP server.
  pub fn new<'a>(address: &'a str) -> std::io::Result<Self> {
    let stream = std::net::TcpStream::connect(address)?;
    return Ok(Self {
      reader: std::io::BufReader::new(stream.try_clone()?),
      writer: std::io::BufWriter::new(stream),
    });
  }
  fn try_parse(&mut self) -> Result<ReturnType, error::Error> {
    use std::io::BufRead;
    let mut line = String::new();
    if let Ok(size) = self.reader.read_line(&mut line) {
      if size == 0 { return Err(error::Error::ReadEmptyData); }
      return match line.chars().nth(0).unwrap() {
        // error or succes, return
        '-' | '+' => Ok(ReturnType::State(line.trim_end().chars().skip(1).collect::<String>())),
        // sized data, parse
        '$' => {
          let size = line
            .chars()
            .skip(1)
            .take_while(|&c| !c.is_whitespace())
            .collect::<String>()
            .parse::<isize>().unwrap();
          if size == -1 {
            return Ok(ReturnType::Null);
          }
          use std::convert::TryInto;
          let mut line = String::new();
          return match self.reader.read_line(&mut line) {
            Ok(n) if n == 0 => Err(error::Error::ReadEmptyData),
            Ok(_) => Ok(ReturnType::BulkString {
              data: line.trim_end().to_string(),
              size: size.try_into().unwrap(),
            }),
            _ => Err(error::Error::FailedDataRead),
          };
        },
        // integer, parse to the end of the line
        ':' => Ok(ReturnType::Integer(line.trim_end().chars().skip(1).collect::<String>())),
        // array of data, can contain anything
        '*' => {
          let mut size = line
            .chars()
            .skip(1)
            .take_while(|&c| !c.is_whitespace())
            .collect::<String>()
            .parse::<isize>().unwrap();
          if size == -1 {
            return Ok(ReturnType::Null);
          }
          if size == 0 {
            return Ok(ReturnType::Array(Vec::new()));
          }
          let mut array_conents: Vec<Box<ReturnType>> = Vec::new();
          while size > 0 {
            array_conents.push(Box::new(self.try_parse()?));
            size -= 1;
          }
          return Ok(ReturnType::Array(array_conents));
        },
        _ => Err(error::Error::UnknownType),
      }
    }
    return Err(error::Error::ReadEmptyData);
  }
  /// Takes redis command to execute and tries to parse the result.
  ///
  /// This is thin wrapper over private `try_parse` function, which does all the parsing.
  pub fn try_execute<'a>(&mut self, command: &'a str) -> Result<ReturnType, error::Error> {
    use crate::write::WriteExt;
    self.writer.write_line(command).unwrap();
    return self.try_parse();
  }
}
