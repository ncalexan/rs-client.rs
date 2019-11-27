use futures::future::join_all;
use rayon::prelude::*;

mod canonical_json;
mod client;
mod kinto_http;
mod signatures;

use client::Client;
use signatures::Verifier;

const SERVER_PROD: &'static str = "https://firefox.settings.services.mozilla.com/v1";

#[tokio::main]
async fn main() {
    let client = Client::new(SERVER_PROD.to_string());

    let entries = client.poll_changes().await.unwrap();

    let futures = entries.iter().map(|(bid, cid, timestamp)| {
        client.fetch_collection(bid.to_owned(), cid.to_owned(), timestamp.to_owned())
    });
    let results = join_all(futures).await;

    let verifier = Verifier::new();
    let failing: Vec<String> = results
        .par_iter()
        .filter_map(|ref result| {
            let dataset = result.as_ref().unwrap();
            if !verifier.verify(&dataset) {
                Some(format!("{}/{}", dataset.bid, dataset.cid))
            } else {
                None
            }
        })
        .collect();
    println!("{:#?}", failing);
}
