use crate::canonical_json::serialize;
use crate::client::RemoteSettingsCollection;

pub struct Verifier {}

impl Verifier {
    pub fn new() -> Self {
        Verifier {}
    }

    pub fn verify(&self, dataset: &RemoteSettingsCollection) -> bool {
        let signature = dataset.metadata["signature"]["signature"]
            .as_str()
            .unwrap()
            .to_string();
        let canonical: Vec<String> = dataset.records.iter().map(|r| serialize(r)).collect();

        let serialized = format!(
            "{{\"data\":[{}],\"last_modified\":{}}}",
            canonical.join(","),
            dataset.timestamp
        );

        signature == serialized
    }
}
