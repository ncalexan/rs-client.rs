use futures::future::join_all;

mod canonical_json;
mod client;
mod kinto_http;
mod signatures;

use client::{Client, Collection};
use signatures::Verifier;
use std::collections::HashMap;

const SERVER_PROD: &'static str = "https://firefox.settings.services.mozilla.com/v1";

#[tokio::main]
async fn main() {
    let client = Client::new(SERVER_PROD.to_string());

    let entries = client.poll_changes().await.unwrap();

    let futures = entries.iter().map(|(bid, cid, timestamp)| {
        client.fetch_collection(bid.to_owned(), cid.to_owned(), timestamp.to_owned())
    });
    let results = join_all(futures).await;
    let datasets: Vec<&Collection> = results
        .iter()
        .map(|ref result| result.as_ref().unwrap())
        .collect();

    let verifier = Verifier::new();
    let verif_futures = datasets.iter().map(|ref dataset| verifier.verify(&dataset));
    let verif_results = join_all(verif_futures).await;

    let failing: HashMap<String, String> = datasets
        .iter()
        .zip(verif_results)
        .filter_map(|(dataset, verif)| {
            Some((
                format!("{}/{}", dataset.bid, dataset.cid),
                match verif {
                    Ok(()) => "OK".to_string(),
                    Err(e) => format!("{:?}", e),
                },
            ))
        })
        .collect();

    println!("{:#?}", failing);
}
