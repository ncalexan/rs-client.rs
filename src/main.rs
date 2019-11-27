use futures::future::join_all;
use serde::Deserialize;
use serde_json;

const SERVER_PROD: &'static str = "https://firefox.settings.services.mozilla.com/v1";

type KintoObject = serde_json::Value;

#[derive(Deserialize)]
struct KintoPluralResponse {
    data: Vec<KintoObject>,
}

#[derive(Deserialize)]
struct KintoObjectResponse {
    data: KintoObject,
}

struct RemoteSettingsCollection {
    bucket: String,
    collection: String,
    metadata: KintoObject,
    records: Vec<KintoObject>,
    timestamp: String,
}

async fn get_collection_metadata(
    server: String,
    bid: String,
    cid: String,
    expected: u64,
) -> Result<KintoObject, reqwest::Error> {
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

async fn get_records(
    server: String,
    bid: String,
    cid: String,
    expected: u64,
) -> Result<(Vec<KintoObject>, String), reqwest::Error> {
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

fn canonical_json(input: &serde_json::value::Value) -> String {
    format!("{:?}", input)
}

fn validate_signature(dataset: &RemoteSettingsCollection) -> bool {
    let signature = dataset.metadata["signature"]["signature"]
        .as_str()
        .unwrap()
        .to_string();
    let canonical: Vec<String> = dataset.records.iter().map(|r| canonical_json(r)).collect();

    let serialized = format!(
        "{{\"data\":[{}],\"last_modified\":{}}}",
        canonical.join(","),
        dataset.timestamp
    );

    signature == serialized
}

#[tokio::main]
async fn main() {
    let (records, timestamp) = get_records(
        SERVER_PROD.to_string(),
        "monitor".to_string(),
        "changes".to_string(),
        0,
    )
    .await
    .unwrap();

    println!("Last modified {}", timestamp);

    let entries: Vec<(String, String, u64)> = records
        .iter()
        .filter_map(|entry| {
            let bid = entry["bucket"].as_str().unwrap().to_string();
            let cid = entry["collection"].as_str().unwrap().to_string();
            let timestamp = entry["last_modified"].as_u64().unwrap();
            if !bid.ends_with("preview") {
                Some((bid, cid, timestamp))
            } else {
                None
            }
        })
        .collect();

    let metadata_futures = entries.iter().map(|(bid, cid, timestamp)| {
        get_collection_metadata(
            SERVER_PROD.to_string(),
            bid.to_owned(),
            cid.to_owned(),
            timestamp.to_owned(),
        )
    });
    let metadata_results = join_all(metadata_futures).await;

    let records_futures = entries.iter().map(|(bid, cid, timestamp)| {
        get_records(
            SERVER_PROD.to_string(),
            bid.to_owned(),
            cid.to_owned(),
            timestamp.to_owned(),
        )
    });
    let records_results = join_all(records_futures).await;

    let all_dataset: Vec<RemoteSettingsCollection> = entries
        .iter()
        .zip(metadata_results)
        .zip(records_results)
        .map(
            |(((bucket, collection, _), ref metadata_result), ref records_result)| {
                let metadata = metadata_result.as_ref().unwrap();
                let (records, timestamp) = records_result.as_ref().unwrap();

                RemoteSettingsCollection {
                    bucket: bucket.to_owned(),
                    collection: collection.to_owned(),
                    metadata: metadata.to_owned(),
                    records: records.to_owned(),
                    timestamp: timestamp.to_owned(),
                }
            },
        )
        .collect();

    for dataset in all_dataset {
        let valid = validate_signature(&dataset);
        println!("{}/{}: {}", dataset.bucket, dataset.collection, valid);
    }
}
