use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};

/// Check if a request will accept JSON, based on the headers.
fn accepts_json(headers: &HeaderMap) -> bool {
    let accept_header = match headers.get("Accept").and_then(|x| x.to_str().ok()) {
        None => return false,
        Some(x) => x,
    };
    // NOTE: this can be improved to properly handle qualities and yadda yadda,
    // but for API consumers, they have the freedom to set their headers appropriately.
    accept_header.starts_with("application/json")
}

/// An extractor for checking of a request will accept JSON.
///
/// Right now this just look for a header that starts with `Accept: application/json`,
/// which isn't technically the only case where a client should be delivered JSON,
/// but is good enough for API consumers, who have the ability to set their own headers.
pub struct AcceptsJson(pub bool);

#[async_trait]
impl<S> FromRequestParts<S> for AcceptsJson
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, ()> {
        Ok(Self(accepts_json(&parts.headers)))
    }
}
