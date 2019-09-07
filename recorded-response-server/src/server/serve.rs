use bytes::Bytes;
use futures::future::poll_fn;
use h2::server;
use h2::server::SendResponse;
use h2::RecvStream;
use h2::SendStream;
use http::header::HeaderName;
use http::header::ACCEPT;
use http::request::Parts;
use http::Method;
use http::Request;
use http::Response;
use http::StatusCode;
use http::Uri;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tracerbench_recorded_response_set::RecordedResponseSet;

static EVENT_STREAM: &[u8] = b"text/event-stream";

/// Serves the H2 connection with the specified response set.
pub(super) async fn serve_h2<S>(socket: S, set: Arc<RecordedResponseSet>) -> Result<(), h2::Error>
where
  S: AsyncRead + AsyncWrite + Unpin,
{
  let mut connection = server::handshake(socket).await?;
  log::debug!("{} HTTP2 connection bound", set.name());
  while let Some(result) = connection.accept().await {
    let (request, send_response) = result?;
    spawn_accept_request(set.clone(), request, send_response);
  }
  Ok(())
}

fn spawn_accept_request(
  response_set: Arc<RecordedResponseSet>,
  request: Request<RecvStream>,
  send_response: SendResponse<Bytes>,
) {
  tokio::spawn(async move {
    let (head, body) = request.into_parts();
    RequestAcceptor::new(response_set, head)
      .accept(body, send_response)
      .await
  });
}

struct RequestAcceptor {
  response_set: Arc<RecordedResponseSet>,
  head: Parts,
}

impl RequestAcceptor {
  fn new(response_set: Arc<RecordedResponseSet>, head: Parts) -> Self {
    RequestAcceptor { head, response_set }
  }

  fn name(&self) -> &str {
    self.response_set.name()
  }

  fn method(&self) -> &Method {
    &self.head.method
  }

  fn uri(&self) -> &Uri {
    &self.head.uri
  }

  fn is_get(&self) -> bool {
    self.head.method == Method::GET
  }

  fn is_head(&self) -> bool {
    self.head.method == Method::HEAD
  }

  fn header_equals(&self, name: HeaderName, value: &[u8]) -> bool {
    if let Some(header) = self.head.headers.get(name) {
      header == value
    } else {
      false
    }
  }

  fn is_server_sent_events(&self) -> bool {
    self.is_get() && self.header_equals(ACCEPT, EVENT_STREAM)
  }

  fn get_response(&self) -> Option<(Response<()>, Option<Bytes>)> {
    let method = if self.is_head() {
      &Method::GET
    } else {
      self.method()
    };
    self.response_set.response_for(method, self.uri())
  }

  async fn accept(&self, body: RecvStream, send_response: SendResponse<Bytes>) {
    log::debug!("{} ACCEPT {} {}", self.name(), self.method(), self.uri());
    if let Err(err) = self.handle(body, send_response).await {
      log::warn!(
        "{} ERROR {} {} {}",
        self.name(),
        self.method(),
        self.uri(),
        err
      );
    }
  }

  async fn handle(
    &self,
    body: RecvStream,
    send_response: SendResponse<Bytes>,
  ) -> Result<(), h2::Error> {
    // server-sent events request we just keep open
    // until the client closes
    // we currently dont support sending any events
    if self.is_server_sent_events() {
      log::debug!("{} Server-Sent Events {}", self.name(), self.uri());
      self.respond_and_wait_for_reset(send_response).await?;
      return Ok(());
    }

    // we want to consume the body before replying
    // even though we don't currently use the body for
    // (has not mattered for initial render benchmarking).
    self.read_body(body).await?;

    self.respond(send_response).await?;

    Ok(())
  }

  async fn read_body(&self, mut body: RecvStream) -> Result<Option<usize>, h2::Error> {
    let mut total = None;
    while let Some(result) = poll_fn(|cx| body.poll_data(cx)).await {
      let chunk = result?;
      total = Some(total.unwrap_or(0) + chunk.len());
    }
    Ok(total)
  }

  async fn respond(&self, send_response: SendResponse<bytes::Bytes>) -> Result<(), h2::Error> {
    if let Some((response, maybe_body)) = self.get_response() {
      if let Some(body) = maybe_body {
        if !self.is_head() {
          return self.respond_with_body(send_response, response, body).await;
        }
      }
      self.respond_with_no_body(send_response, response)
    } else {
      self.respond_with_not_found(send_response)
    }
  }

  async fn respond_with_body(
    &self,
    mut respond: SendResponse<Bytes>,
    response: Response<()>,
    body: Bytes,
  ) -> Result<(), h2::Error> {
    let status = response.status().as_u16();

    let send_stream = respond.send_response(response, false)?;
    let sent = self.send_body(send_stream, body).await?;

    log::debug!(
      "{} {} {} {} {}",
      self.name(),
      status,
      self.method(),
      self.uri(),
      sent
    );

    Ok(())
  }

  fn respond_with_no_body(
    &self,
    mut respond: SendResponse<Bytes>,
    response: Response<()>,
  ) -> Result<(), h2::Error> {
    let status = response.status().as_u16();

    respond.send_response(response, true)?;

    log::debug!(
      "{} {} {} {} None",
      self.name(),
      status,
      self.method(),
      self.uri()
    );

    Ok(())
  }

  fn respond_with_not_found(&self, mut respond: SendResponse<Bytes>) -> Result<(), h2::Error> {
    let mut response = Response::new(());
    *response.status_mut() = StatusCode::NOT_FOUND;

    respond.send_response(response, true)?;

    log::debug!("{} 404 {} {} None", self.name(), self.method(), self.uri());

    Ok(())
  }

  async fn respond_and_wait_for_reset(
    &self,
    mut respond: SendResponse<Bytes>,
  ) -> Result<(), h2::Error> {
    let mut send_stream = respond.send_response(Response::new(()), false)?;
    poll_fn(|cx| send_stream.poll_reset(cx)).await?;
    Ok(())
  }

  async fn send_body(
    &self,
    mut send_stream: SendStream<Bytes>,
    mut body: Bytes,
  ) -> Result<usize, h2::Error> {
    let total = body.len();

    send_stream.reserve_capacity(total);

    let mut available = send_stream.capacity();

    while available < body.len() {
      if available > 0 {
        send_stream.send_data(body.split_to(available), false)?;
      }

      available = match poll_fn(|cx| send_stream.poll_capacity(cx)).await {
        Some(Ok(n)) => n,
        Some(Err(err)) => return Err(err),
        None => return Ok(total), // no longer streaming
      }
    }

    send_stream.send_data(body, true)?;

    Ok(total)
  }
}
