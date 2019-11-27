use crate::kinto_http::{get_collection_metadata, get_records, KintoError, KintoObject};

#[derive(Debug)]
pub enum ClientError {}

impl From<KintoError> for ClientError {
    fn from(err: KintoError) -> Self {
        err.into()
    }
}

pub struct RemoteSettingsCollection {
    pub bid: String,
    pub cid: String,
    pub metadata: KintoObject,
    pub records: Vec<KintoObject>,
    pub timestamp: String,
}

pub struct Client {
    server: String,
}

impl Client {
    pub fn new(server: String) -> Self {
        Client {
            server: server.to_owned(),
        }
    }

    pub async fn poll_changes(&self) -> Result<Vec<(String, String, u64)>, ClientError> {
        let (records, _) = get_records(
            self.server.to_owned(),
            "monitor".to_string(),
            "changes".to_string(),
            0,
        )
        .await
        .unwrap();

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

        Ok(entries)
    }

    pub async fn fetch_collection(
        &self,
        bid: String,
        cid: String,
        expected: u64,
    ) -> Result<RemoteSettingsCollection, ClientError> {
        let metadata = get_collection_metadata(
            self.server.to_owned(),
            bid.to_owned(),
            cid.to_owned(),
            expected,
        )
        .await?;
        let (records, timestamp) = get_records(
            self.server.to_owned(),
            bid.to_owned(),
            cid.to_owned(),
            expected,
        )
        .await?;

        Ok(RemoteSettingsCollection {
            bid: bid.to_owned(),
            cid: cid.to_owned(),
            metadata: metadata.to_owned(),
            records: records.to_owned(),
            timestamp: timestamp.to_owned(),
        })
    }
}
