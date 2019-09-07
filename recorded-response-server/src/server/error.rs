use std::error;
use std::fmt;
use std::io;
use tokio_rustls::rustls::TLSError;
use tracerbench_socks_proxy::SocksError;

#[derive(Debug)]
pub(super) enum ServerError {
  IO(io::Error),
  Socks(SocksError),
  TLS(TLSError),
  H2(h2::Error),
}

impl From<io::Error> for ServerError {
  fn from(err: io::Error) -> ServerError {
    ServerError::IO(err)
  }
}

impl From<SocksError> for ServerError {
  fn from(err: SocksError) -> ServerError {
    ServerError::Socks(err)
  }
}

impl From<TLSError> for ServerError {
  fn from(err: TLSError) -> ServerError {
    ServerError::TLS(err)
  }
}

impl From<h2::Error> for ServerError {
  fn from(err: h2::Error) -> ServerError {
    ServerError::H2(err)
  }
}

impl error::Error for ServerError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match *self {
      ServerError::IO(ref err) => error::Error::source(err),
      ServerError::Socks(ref err) => error::Error::source(err),
      ServerError::TLS(ref err) => error::Error::source(err),
      ServerError::H2(ref err) => error::Error::source(err),
    }
  }
}

impl fmt::Display for ServerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match *self {
      ServerError::IO(ref err) => err.fmt(f),
      ServerError::Socks(ref err) => err.fmt(f),
      ServerError::TLS(ref err) => err.fmt(f),
      ServerError::H2(ref err) => err.fmt(f),
    }
  }
}
