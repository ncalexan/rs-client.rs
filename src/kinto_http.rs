use serde::Deserialize;
use serde_json;

use url::{ParseError, Url};
use viaduct::{Error as ViaductError, Request};

pub type KintoObject = serde_json::Value;

#[derive(Deserialize)]
struct KintoPluralResponse {
    data: Vec<KintoObject>,
}

#[derive(Deserialize)]
pub struct ChangesetResponse {
    pub metadata: KintoObject,
    pub changes: Vec<KintoObject>,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum KintoError {
    Error,
}

impl From<ViaductError> for KintoError {
    fn from(err: ViaductError) -> Self {
        err.into()
    }
}

impl From<serde_json::Error> for KintoError {
    fn from(err: serde_json::Error) -> Self {
        err.into()
    }
}

impl From<ParseError> for KintoError {
    fn from(err: ParseError) -> Self {
        err.into()
    }
}

pub fn get_records(
    server: String,
    bid: String,
    cid: String,
    expected: u64,
) -> Result<(Vec<KintoObject>, String), KintoError> {
    let url = format!(
        "{}/buckets/{}/collections/{}/records?_expected={}",
        server, bid, cid, expected
    );
    println!("Fetch {}...", url);
    let resp = Request::get(Url::parse(&url)?).send()?;
    let timestamp = resp
        .headers
        .get("etag").ok_or(KintoError::Error)?
        .to_string();
    let size = resp.headers.get("content-length").ok_or(KintoError::Error)?;
    println!("Download {:?} bytes...", size);
    let result: KintoPluralResponse = resp.json()?;

    Ok((result.data, timestamp))
}

pub fn get_changeset(
    server: String,
    bid: String,
    cid: String,
    expected: u64,
) -> Result<ChangesetResponse, KintoError> {
    let url = format!(
        "{}/buckets/{}/collections/{}/changeset?_expected={}",
        server, bid, cid, expected
    );
    println!("Fetch {}...", url);
    let resp = Request::get(Url::parse(&url)?).send()?;
    let result: ChangesetResponse = resp.json()?;

    Ok(result)
}

