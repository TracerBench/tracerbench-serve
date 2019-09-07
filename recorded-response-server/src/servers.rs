use super::Config;
use super::Server;
use futures::future::try_join_all;
use std::io;
use std::ops::Deref;
use std::sync::Arc;
use tokio_rustls::rustls;
use tracerbench_recorded_response_set::RecordedResponseSets;

pub struct Servers(Vec<Server>);

impl Deref for Servers {
  type Target = Vec<Server>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Config> for Servers {
  fn from(config: Config) -> Self {
    Self::from_config(config)
  }
}

impl Servers {
  pub fn from_parts(
    tls_config: Arc<rustls::ServerConfig>,
    mut response_sets: RecordedResponseSets,
  ) -> Self {
    let mut servers: Vec<Server> = Vec::with_capacity(response_sets.len());
    for response_set in response_sets.drain(..) {
      servers.push(Server::new(tls_config.clone(), response_set));
    }
    Servers(servers)
  }

  pub fn from_config(config: Config) -> Self {
    Self::from_parts(config.tls_config, config.response_sets)
  }

  pub async fn start(&self) -> Result<(), io::Error> {
    let mut futures = Vec::with_capacity(self.len());
    for server in self.iter() {
      futures.push(server.start());
    }
    try_join_all(futures).await?;
    Ok(())
  }
}
