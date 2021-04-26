mod error;
mod serve;

use error::ServerError;
use serve::serve_h2;
use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_rustls::rustls;
use tokio_rustls::TlsAcceptor;
use tracerbench_recorded_response_set::RecordedResponseSet;
use tracerbench_socks_proxy::socks5_handshake;

/// Server listens on a port with a socks proxy -> tls -> h2
/// serving recorded responses from a set
pub struct Server {
  tls_config: Arc<rustls::ServerConfig>,
  response_set: Arc<RecordedResponseSet>,
}

impl Server {
  pub fn new(
    tls_config: Arc<rustls::ServerConfig>,
    response_set: Arc<RecordedResponseSet>,
  ) -> Self {
    Server {
      tls_config,
      response_set,
    }
  }

  pub fn name(&self) -> &str {
    self.response_set.name()
  }

  pub fn addr(&self) -> SocketAddr {
    SocketAddr::new(
      Ipv4Addr::new(127, 0, 0, 1).into(),
      self.response_set.socks_port(),
    )
  }

  pub async fn start(&self) -> Result<(), io::Error> {
    let listener = TcpListener::bind(self.addr()).await?;

    println!(
      "response set {} socks proxy server listening at {}",
      self.name(),
      self.addr()
    );

    loop {
      match listener.accept().await {
        Ok((socket, _peer_addr)) => self.spawn(socket),
        Err(err) => log::warn!("failed to accept client {:?}", err),
      }
    }
  }

  fn spawn(&self, socket: TcpStream) {
    let tls_config = self.tls_config.clone();
    let response_set = self.response_set.clone();
    tokio::spawn(async move {
      if let Err(err) = handle_tcp_connection(socket, tls_config, response_set).await {
        log::warn!("{:?}", err);
      }
    });
  }
}

async fn handle_tcp_connection(
  socket: TcpStream,
  tls_config: Arc<rustls::ServerConfig>,
  response_set: Arc<RecordedResponseSet>,
) -> Result<(), ServerError> {
  let socket = socks5_handshake(socket).await?;
  let tls_socket = TlsAcceptor::from(tls_config).accept(socket).await?;
  serve_h2(tls_socket, response_set).await?;
  Ok(())
}
