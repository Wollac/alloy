use crate::Http;
use alloy_json_rpc::{RequestPacket, ResponsePacket};
use alloy_transport::{TransportError, TransportErrorKind, TransportFut};
use hyper::{
    body::Bytes,
    client::{connect::Connect, Client},
};
use std::task;
use tower::Service;

impl<C> Http<Client<C>>
where
    C: Connect + Clone + Send + Sync + 'static,
{
    /// Make a request.
    fn request(&self, req: RequestPacket) -> TransportFut<'static> {
        let this = self.clone();
        Box::pin(async move {
            let ser = req.serialize().map_err(TransportError::ser_err)?;

            // convert the Box<RawValue> into a hyper request<B>
            let body: Box<str> = ser.into();
            let body: Box<[u8]> = body.into();
            let req = hyper::Request::builder()
                .method(hyper::Method::POST)
                .uri(this.url.as_str())
                .header("content-type", "application/json")
                .body(hyper::Body::from(Bytes::from(body)))
                .expect("request parts are valid");

            let resp = this.client.request(req).await.map_err(TransportErrorKind::custom)?;

            let status = resp.status();

            // unpack data from the response body. We do this regardless of
            // the status code, as we want to return the error in the body if
            // there is one.
            let body = hyper::body::to_bytes(resp.into_body())
                .await
                .map_err(TransportErrorKind::custom)?;

            if status != hyper::StatusCode::OK {
                return Err(TransportErrorKind::custom_str(&format!(
                    r#"HTTP error: {} with body: "{}""#,
                    status,
                    String::from_utf8_lossy(body.as_ref())
                )));
            }

            // Deser a Box<RawValue> from the body. If deser fails, return the
            // body as a string in the error. If the body is not UTF8, this will
            // fail and give the empty string in the error.
            serde_json::from_slice(body.as_ref()).map_err(|err| {
                TransportError::deser_err(err, std::str::from_utf8(body.as_ref()).unwrap_or(""))
            })
        })
    }
}

impl<C> Service<RequestPacket> for &Http<Client<C>>
where
    C: Connect + Clone + Send + Sync + 'static,
{
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        // hyper always returns ok
        task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: RequestPacket) -> Self::Future {
        self.request(req)
    }
}

impl<C> Service<RequestPacket> for Http<Client<C>>
where
    C: Connect + Clone + Send + Sync + 'static,
{
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        // hyper always returns ok
        task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: RequestPacket) -> Self::Future {
        self.request(req)
    }
}
