use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use serde::{de::DeserializeOwned, Deserialize};

/// Represents a type which can be used as a cursor, to make a sub-selection of items.
pub trait Cursor: DeserializeOwned {
    /// The minimum value for the cursor.
    const MIN: Self;
    /// The maximum value for the cursor.
    const MAX: Self;

    /// The number of items between two points in the cursor
    fn distance_between(&self, other: &Self) -> u64;
}

impl Cursor for i64 {
    const MIN: Self = 0;
    const MAX: Self = i64::MAX;

    fn distance_between(&self, other: &Self) -> u64 {
        self.overflowing_sub(*other).0.unsigned_abs() + 1
    }
}

// This type exist only to allow for using axum::Query<_> to parse it.
#[derive(Debug, Clone, Deserialize)]
struct RawPagination<C> {
    start: Option<C>,
    stop: Option<C>,
}

impl<C: Cursor> Default for RawPagination<C> {
    fn default() -> Self {
        Self {
            start: None,
            stop: None,
        }
    }
}

/// An extractor for getting pagination for a particular cursor type.
///
/// This will parse the pagination options from the query params in the URL.
#[derive(Debug, Clone)]
pub struct Pagination<C> {
    pub start: C,
    pub stop: C,
}

impl<C: Cursor> From<RawPagination<C>> for Pagination<C> {
    fn from(value: RawPagination<C>) -> Self {
        Self {
            start: value.start.unwrap_or(C::MIN),
            stop: value.stop.unwrap_or(C::MAX),
        }
    }
}

#[allow(dead_code)]
impl<C: Cursor + tracing::Value> Pagination<C> {
    pub fn limit(&self, no_more_than: u64) -> u64 {
        u64::min(C::distance_between(&self.start, &self.stop), no_more_than)
    }
}

#[async_trait]
impl<S, C: Cursor> FromRequestParts<S> for Pagination<C>
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, ()> {
        Ok(Query::<RawPagination<C>>::try_from_uri(&parts.uri)
            .map(|x| x.0)
            .unwrap_or(RawPagination::default())
            .into())
    }
}
