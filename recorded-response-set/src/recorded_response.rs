use bytes::Bytes;
use http::{HeaderMap, Response, StatusCode};
use std::sync::Arc;

/// cheap to clone structure of response data
#[derive(Debug, Clone)]
pub struct RecordedResponse {
  status_code: StatusCode,
  headers: Arc<HeaderMap>,
  body: Option<Bytes>,
}

impl RecordedResponse {
  pub fn to_parts(&self) -> (Response<()>, Option<Bytes>) {
    let mut response = Response::new(());
    *response.status_mut() = self.status_code;
    *response.headers_mut() = self.headers.as_ref().clone();
    let body = self.body.clone();
    (response, body)
  }
}

impl RecordedResponse {
  pub fn new(status_code: StatusCode, headers: Arc<HeaderMap>, body: Option<Bytes>) -> Self {
    RecordedResponse {
      status_code,
      headers,
      body,
    }
  }

  pub fn status_code(&self) -> StatusCode {
    self.status_code
  }

  pub fn headers(&self) -> &HeaderMap {
    self.headers.as_ref()
  }

  pub fn body(&self) -> Option<&Bytes> {
    self.body.as_ref()
  }
}
