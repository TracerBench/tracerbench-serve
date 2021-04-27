use memmap::Mmap;
use ring::digest::digest;
use ring::digest::SHA256;
use std::error;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind};
use std::path::PathBuf;
use std::sync::Arc;
use tokio_rustls::rustls::internal::pemfile;
use tokio_rustls::rustls::{Certificate, NoClientAuth, PrivateKey, ServerConfig};
use tracerbench_recorded_response_set::RecordedResponseSets;
use webpki::TrustAnchor;

static ALPN_H2: &[u8] = b"h2";

pub(super) fn build_tls_config(
  certs: Vec<Certificate>,
  key: PrivateKey,
) -> Result<Arc<ServerConfig>, Error> {
  let mut config = ServerConfig::new(NoClientAuth::new());
  config.set_single_cert(certs, key).map_err(invalid_input)?;
  config.alpn_protocols.push(ALPN_H2.to_vec());
  Ok(Arc::new(config))
}

pub(super) fn read_cert_pem(path: &PathBuf) -> Result<Vec<Certificate>, Error> {
  let file = File::open(path)?;
  let mut reader = BufReader::new(&file);
  if let Ok(certs) = pemfile::certs(&mut reader) {
    if certs.is_empty() {
      Err(missing_certificate(path))
    } else {
      Ok(certs)
    }
  } else {
    Err(invalid_pem_file(path))
  }
}

pub(super) fn read_key_pem(path: &PathBuf) -> Result<PrivateKey, Error> {
  let file = File::open(path)?;
  let mut reader = BufReader::new(&file);
  if let Ok(mut private_keys) = pemfile::rsa_private_keys(&mut reader) {
    if private_keys.is_empty() {
      Err(missing_rsa_private_key(path))
    } else {
      Ok(private_keys.remove(0))
    }
  } else {
    Err(invalid_pem_file(path))
  }
}

pub(super) fn read_response_set_cbor(path: &PathBuf) -> Result<RecordedResponseSets, Error> {
  let file = File::open(path)?;
  let mmap = unsafe { Mmap::map(&file)? };
  let response_sets = serde_cbor::from_slice(&mmap).map_err(invalid_data)?;
  Ok(response_sets)
}

/// The base64 encoded SHA256 digest of subject public key info.
/// This is needed for the chrome command line switch ignore-certificate-errors-spki-list
pub(super) fn spki_digest(cert_der: &[u8]) -> Result<String, Error> {
  let trust_anchor = TrustAnchor::try_from_cert_der(cert_der).map_err(invalid_data)?;
  // unfortunately the only public api exposed to get the subjectPublicKeyInfo
  // is this one, and it is only the V part of the DER TLV
  // and we need the TL part
  let value_len = trust_anchor.spki.len();
  let be_bytes = usize::to_be_bytes(value_len);
  let start = be_bytes.iter().position(|&b| b > 0).unwrap();
  let length_bytes = &be_bytes[start..];

  let length_byte_length = length_bytes.len() as u8;
  let len_127_or_less = length_byte_length == 1 && length_bytes[0] < 0x80;
  let capacity = if len_127_or_less {
    value_len + 2
  } else {
    value_len + 2 + length_byte_length as usize
  };

  let mut seq_tlv: Vec<u8> = Vec::with_capacity(capacity);

  // SEQUENCE TAG
  seq_tlv.push(0x30);
  if len_127_or_less {
    // if length is < 128 store length directly
    seq_tlv.push(length_bytes[0])
  } else {
    // if length is >= 128 set the hi bit and the byte length of the length
    seq_tlv.push(length_byte_length | 0x80);
    // add the length bytes
    seq_tlv.extend_from_slice(length_bytes);
  }

  // add the value
  seq_tlv.extend_from_slice(trust_anchor.spki);

  let spki_digest = digest(&SHA256, &seq_tlv);
  Ok(base64::encode(spki_digest.as_ref()))
}

fn missing_certificate(path: &PathBuf) -> Error {
  invalid_input(format!(
    "Missing CERTIFICATE section in PEM file: {:?}",
    path
  ))
}

fn missing_rsa_private_key(path: &PathBuf) -> Error {
  invalid_input(format!(
    "Missing RSA PRIVATE KEY section in PEM file: {:?}",
    path
  ))
}

fn invalid_pem_file(path: &PathBuf) -> Error {
  invalid_data(format!("Invalid PEM file: {:?}", path))
}

fn invalid_data<E>(err: E) -> Error
where
  E: Into<Box<dyn error::Error + Send + Sync>>,
{
  Error::new(ErrorKind::InvalidData, err)
}

fn invalid_input<E>(err: E) -> Error
where
  E: Into<Box<dyn error::Error + Send + Sync>>,
{
  Error::new(ErrorKind::InvalidInput, err)
}
