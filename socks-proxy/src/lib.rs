#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

use log::debug;
use std::error;
use std::fmt;
use std::io;
use std::net;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

const SOCKS_VERSION: u8 = b'\x05';
const NO_AUTHENTICATION_REQUIRED: u8 = b'\x00';
const NO_ACCEPTABLE_METHODS: u8 = b'\xFF';
const COMMAND_NOT_SUPPORTED: u8 = b'\x07';
const ADDRESS_TYPE_NOT_SUPPORTED: u8 = b'\x08';
const CONNECT_COMMAND: u8 = b'\x01';
const ADDRESS_TYPE_IPV4: u8 = b'\x01';
const ADDRESS_TYPE_DOMAIN_NAME: u8 = b'\x03';
const ADDRESS_TYPE_IPV6: u8 = b'\x04';

// +----+--------+
// |VER | METHOD |
// +----+--------+
// | 1  |   1    |
// +----+--------+
static NO_AUTHENTICATION_REQUIRED_REPLY: [u8; 2] = [SOCKS_VERSION, NO_AUTHENTICATION_REQUIRED];
static NO_ACCEPTABLE_METHODS_REPLY: [u8; 2] = [SOCKS_VERSION, NO_ACCEPTABLE_METHODS];

// +----+-----+-------+------+----------+----------+
// |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
// +----+-----+-------+------+----------+----------+
// | 1  |  1  | X'00' |  1   | Variable |    2     |
// +----+-----+-------+------+----------+----------+
static CONNECT_REPLY: [u8; 10] = [SOCKS_VERSION, 0, 0, ADDRESS_TYPE_IPV4, 0, 0, 0, 0, 0, 0];
static COMMAND_NOT_SUPPORTED_REPLY: [u8; 2] = [SOCKS_VERSION, COMMAND_NOT_SUPPORTED];
static ADDRESS_TYPE_NOT_SUPPORTED_REPLY: [u8; 2] = [SOCKS_VERSION, ADDRESS_TYPE_NOT_SUPPORTED];

#[derive(Debug)]
pub enum SocksError {
  InvalidSocksVersion(u8),
  NoAcceptableMethods,
  CommandNotSupported(u8),
  AddressTypeNotSupported(u8),
  IOError(io::Error),
}

/// This only supports no authentication and just accepts
/// with a reply of success with 0.0.0.0:0
///
/// It is simply for redirecting all trafic to the local server
pub async fn socks5_handshake<S>(mut socket: S) -> Result<S, SocksError>
where
  S: AsyncReadExt + AsyncWriteExt + Unpin,
{
  // max possible socks message
  // 4 header 1 domain length 255 domain name 2 port
  let mut buffer: [u8; 262] = [0; 262];

  // +----+----------+----------+
  // |VER | NMETHODS | METHODS  |
  // +----+----------+----------+
  // | 1  |    1     | 1 to 255 |
  // +----+----------+----------+
  let mut len = read_at_least_one(&mut socket, &mut buffer).await?;
  let version = buffer[0];
  if version != SOCKS_VERSION {
    return Err(SocksError::InvalidSocksVersion(version));
  }

  if len < 2 {
    len += read_at_least_one(&mut socket, &mut buffer[len..]).await?;
  }

  let methods_len = buffer[1] as usize;

  let end: usize = methods_len + 2;
  while len < end {
    len += read_at_least_one(&mut socket, &mut buffer[len..]).await?;
  }

  let methods = &buffer[2..end];
  let no_auth_required = methods.iter().any(|&b| b == NO_AUTHENTICATION_REQUIRED);

  if no_auth_required {
    socket.write_all(&NO_AUTHENTICATION_REQUIRED_REPLY).await?;
  } else {
    socket.write_all(&NO_ACCEPTABLE_METHODS_REPLY).await?;
    return Err(SocksError::NoAcceptableMethods);
  }

  // +----+-----+-------+------+----------+----------+
  // |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
  // +----+-----+-------+------+----------+----------+
  // | 1  |  1  | X'00' |  1   | Variable |    2     |
  // +----+-----+-------+------+----------+----------+
  // reset buffer position
  len = read_at_least_one(&mut socket, &mut buffer).await?;

  let version = buffer[0];
  if version != SOCKS_VERSION {
    return Err(SocksError::InvalidSocksVersion(version));
  }

  if len < 2 {
    len += read_at_least_one(&mut socket, &mut buffer[len..]).await?;
  }

  let command = buffer[1];
  if command != CONNECT_COMMAND {
    socket.write_all(&COMMAND_NOT_SUPPORTED_REPLY).await?;
    return Err(SocksError::CommandNotSupported(command));
  }

  while len < 4 {
    len += read_at_least_one(&mut socket, &mut buffer[len..]).await?;
  }

  let address_type = buffer[3];

  // now we have read enough to know the message len
  let end = match address_type {
    ADDRESS_TYPE_IPV4 => 4 + 4 + 2,
    ADDRESS_TYPE_DOMAIN_NAME => {
      if len < 5 {
        len += read_at_least_one(&mut socket, &mut buffer[len..]).await?;
      }
      let domain_name_len = buffer[4] as usize;
      4 + 1 + domain_name_len + 2
    }
    ADDRESS_TYPE_IPV6 => 4 + 16 + 2,
    _ => {
      socket.write_all(&ADDRESS_TYPE_NOT_SUPPORTED_REPLY).await?;
      return Err(SocksError::AddressTypeNotSupported(address_type));
    }
  };

  while len < end {
    len += read_at_least_one(&mut socket, &mut buffer[len..]).await?;
  }

  debug!(
    "CONNECT {}",
    match address_type {
      ADDRESS_TYPE_IPV4 => format!(
        "{}",
        net::SocketAddrV4::new(
          net::Ipv4Addr::new(buffer[4], buffer[5], buffer[6], buffer[7]),
          u16::from_be_bytes([buffer[8], buffer[9]]),
        )
      ),
      ADDRESS_TYPE_DOMAIN_NAME => format!(
        "{}:{}",
        String::from_utf8_lossy(&buffer[5..end - 2]),
        u16::from_be_bytes([buffer[end - 2], buffer[end - 1]])
      ),
      ADDRESS_TYPE_IPV6 => format!(
        "{}",
        net::SocketAddrV6::new(
          net::Ipv6Addr::from([
            buffer[4], buffer[5], buffer[6], buffer[7], buffer[8], buffer[9], buffer[10],
            buffer[11], buffer[12], buffer[13], buffer[14], buffer[15], buffer[16], buffer[17],
            buffer[18], buffer[19],
          ]),
          u16::from_be_bytes([buffer[20], buffer[21]]),
          0,
          0
        )
      ),
      _ => String::default(),
    }
  );

  // we are capturing all traffic so we don't care about what we
  // read just as long as we read it all
  socket.write_all(&CONNECT_REPLY).await?;

  Ok(socket)
}

async fn read_at_least_one<S>(socket: &mut S, buffer: &mut [u8]) -> Result<usize, io::Error>
where
  S: AsyncReadExt + Unpin,
{
  let len = socket.read(buffer).await?;
  if len == 0 {
    return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
  }
  Ok(len)
}

impl error::Error for SocksError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match *self {
      SocksError::IOError(ref err) => error::Error::source(err),
      _ => None,
    }
  }
}

impl fmt::Display for SocksError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match *self {
      SocksError::InvalidSocksVersion(version) => f.write_fmt(format_args!(
        "Invalid socks version, expected 5 but got {}",
        version
      )),
      SocksError::IOError(ref err) => f.write_fmt(format_args!(
        "an IO error occurred during the socks 5 handshake: {}",
        err
      )),
      SocksError::NoAcceptableMethods => {
        f.write_str("no acceptable socks auth methods, only no authentication is supported")
      }
      SocksError::CommandNotSupported(ref cmd) => {
        f.write_fmt(format_args!("unsupported socks command {}", cmd))
      }
      SocksError::AddressTypeNotSupported(ref atype) => {
        f.write_fmt(format_args!("unsupported address type {}", atype))
      }
    }
  }
}

impl From<io::Error> for SocksError {
  fn from(err: io::Error) -> SocksError {
    SocksError::IOError(err)
  }
}
