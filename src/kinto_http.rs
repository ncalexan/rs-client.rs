use serde::Deserialize;
use serde_json;

use reqwest::Error as ReqwestError;

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
pub enum KintoError {}

impl From<ReqwestError> for KintoError {
    fn from(err: ReqwestError) -> Self {
        err.into()
    }
}

pub async fn get_records(
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
    let resp = reqwest::get(&url).await?;
    let timestamp = resp
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let size = resp.headers().get("content-length").unwrap();
    println!("Download {:?} bytes...", size);
    let body = resp.text().await?;
    let result: KintoPluralResponse = serde_json::from_str(&body).unwrap();

    Ok((result.data, timestamp))
}

pub async fn get_changeset(
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
    let resp = reqwest::get(&url).await?;
    let body = resp.text().await?;
    let result: ChangesetResponse = serde_json::from_str(&body).unwrap();

    Ok(result)
}

