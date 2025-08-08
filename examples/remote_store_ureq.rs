use std::io::Error;
use std::result::Result;

use bytes::Bytes;

use perfume::identity::{ConnectionBridge, Population, RemoteStore};

mod common;
use common::test_server;

// generated for this example with `TMP_DIR=/tmp cargo run -F codegen`
include!(concat!(env!("TMP_DIR"), "/perfume.rs"));

const BHUTANESE: Population = Population {
    domain: "bt",
    secret: *b"3D5aPzC0jwT25eAWlEa4FcW8d9FNz00g", // 32 bytes for keyed hasher
    ingredients: &PERFUME_INGREDIENTS,            // see build.rs example below
};

fn main() {
    let _server_handle = test_server("127.0.0.1:9090");

    let mut store = RemoteStore {
        bridge: ExampleBridge {
            url: "http://localhost:9090".try_into().unwrap(),
            domain: BHUTANESE.domain.to_string(),
        },
    };

    let user1 = BHUTANESE.identity("flying@wom.bt", &mut store).unwrap();
    let user2 = BHUTANESE.identity("fast@serpent.bt", &mut store).unwrap();
    let user3 = BHUTANESE.identity("yogi@garbha.bt", &mut store).unwrap();
    assert_eq!(user1.friendly_name, "unraking-teal-muskrat");
    assert_eq!(user2.friendly_name, "outpleasing-rose-gelding");
    assert_eq!(user3.friendly_name, "reifying-navy-lab");

    assert_eq!(
        BHUTANESE.identity("flying@wom.bt", &mut store).unwrap(),
        user1
    );

    // storage is based on the 64 character hash output of the identifier "flying@wom.bt"
    let stored_blob = store
        .bridge
        .get(user1.storage.key.as_str()) // storage key is the first 3 characters of the hash
        .unwrap()
        .unwrap();
    assert_eq!(
        stored_blob.as_ref(),
        // first line of the blob is the last 61 characters of the hash,
        // followed by an offset into a list of random names
        [user1.storage.digest.as_str().as_bytes(), b" 0"].concat()
    );
}

struct ExampleBridge {
    url: http::Uri,
    domain: String,
}

impl ConnectionBridge for ExampleBridge {
    fn get(&self, key: &str) -> Result<Option<Bytes>, Error> {
        let resource_url = format!("{}{}/{}", self.url, self.domain, key);
        let response = ureq::get(&resource_url)
            .config()
            .http_status_as_error(false)
            .build()
            .call()
            .map_err(|e| Error::other(format!("IO failure on request to {resource_url}: {e}")))?;
        match response.status() {
            http::StatusCode::OK => {
                let body = response.into_body().read_to_vec().map_err(|e| {
                    Error::other(format!(
                        "error parsing response body on request to {resource_url}: {e}"
                    ))
                })?;
                Ok(Some(Bytes::from(body)))
            }
            http::StatusCode::NOT_FOUND => Ok(None),
            unexpected => Err(Error::other(format!(
                "unexpected HTTP response on request to {resource_url}: {unexpected}"
            ))),
        }
    }

    fn put(&self, key: &str, body: Bytes) -> Result<(), Error> {
        let resource_url = format!("{}{}/{}", self.url, self.domain, key);
        let response = ureq::put(&resource_url)
            .config()
            .http_status_as_error(false)
            .build()
            .send(&body[..])
            .map_err(|e| Error::other(format!("IO failure on request to {resource_url}: {e}")))?;
        match response.status() {
            http::StatusCode::OK => Ok(()),
            unexpected => Err(Error::other(format!(
                "unexpected HTTP response on request to {resource_url}: {unexpected}"
            ))),
        }
    }

    async fn get_async(&self, _key: &str) -> Result<Option<Bytes>, Error> {
        unimplemented!()
    }

    async fn put_async(&self, _key: &str, _body: Bytes) -> Result<(), Error> {
        unimplemented!()
    }
}
