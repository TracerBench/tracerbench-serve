mod util;

use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use tokio_rustls::rustls;
use tracerbench_recorded_response_set::RecordedResponseSets;
use util::*;

/// Server config
pub struct Config {
  /// Value for chrome switch ignore-certificate-errors-spki-list
  /// BASE64(SHA256(cert.subjectPublicKeyInfo)))
  pub spki_digest: String,
  /// Shared config for a TLS acceptor
  pub tls_config: Arc<rustls::ServerConfig>,
  /// Recorded response sets
  pub response_sets: RecordedResponseSets,
}

impl Config {
  pub fn new(
    spki_digest: String,
    tls_config: Arc<rustls::ServerConfig>,
    response_sets: RecordedResponseSets,
  ) -> Self {
    Config {
      spki_digest,
      tls_config,
      response_sets,
    }
  }

  pub fn from_parts(
    cert_chain: Vec<rustls::Certificate>,
    private_key: rustls::PrivateKey,
    response_sets: RecordedResponseSets,
  ) -> Result<Self, io::Error> {
    Ok(Self::new(
      spki_digest(cert_chain[0].as_ref())?,
      build_tls_config(cert_chain, private_key)?,
      response_sets,
    ))
  }

  pub fn from_args(
    cert_pem: &PathBuf,
    key_pem: &PathBuf,
    response_sets_cbor: &PathBuf,
  ) -> Result<Self, io::Error> {
    Self::from_parts(
      read_cert_pem(cert_pem)?,
      read_key_pem(key_pem)?,
      read_response_set_cbor(response_sets_cbor)?,
    )
  }
}
