use serde::Deserialize;
use serde_json;

use reqwest::Error as ReqwestError;

pub type KintoObject = serde_json::Value;

#[derive(Deserialize)]
struct KintoPluralResponse {
    data: Vec<KintoObject>,
}

#[derive(Deserialize)]
struct KintoObjectResponse {
    data: KintoObject,
}

#[derive(Debug)]
pub enum KintoError {}

impl From<ReqwestError> for KintoError {
    fn from(err: ReqwestError) -> Self {
        err.into()
    }
}

pub async fn get_collection_metadata(
    server: String,
    bid: String,
    cid: String,
    expected: u64,
) -> Result<KintoObject, KintoError> {
    let url = format!(
        "{}/buckets/{}/collections/{}?_expected={}",
        server, bid, cid, expected
    );
    println!("Fetch {}...", url);
    let resp = reqwest::get(&url).await?;
    let body = resp.text().await?;
    let result: KintoObjectResponse = serde_json::from_str(&body).unwrap();

    Ok(result.data)
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
