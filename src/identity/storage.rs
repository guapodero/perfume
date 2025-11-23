use std::io::BufRead;

use async_generic::async_generic;
use bytes::Bytes;
use std::future::Future;

use crate::hex_string::HexString;
use crate::{STORAGE_DIGEST_LENGTH, STORAGE_KEY_LENGTH};

/// Persisted identity data necessary to implement [`StorageState`].
#[derive(Debug, Clone)]
pub struct Storage {
    /// Used to determine the first word of a friendly name.
    pub key: HexString<STORAGE_KEY_LENGTH>,
    /// A per-identity object hash, used to determine the last two words of a friendly name.
    pub digest: HexString<STORAGE_DIGEST_LENGTH>,
}

impl From<&[u8]> for Storage {
    fn from(value: &[u8]) -> Self {
        Self {
            key: value[..STORAGE_KEY_LENGTH].into(),
            digest: value[STORAGE_KEY_LENGTH..].into(),
        }
    }
}

/// Persistence scheme for [`Storage`] objects.
/// At least one of the required methods should be implemented.
pub trait StorageState {
    /// Defines a chronological ordering of `Storage` objects based on when they were first stored.
    /// For each `storage` argument, a unique *persisted* offset should be returned.
    /// For each `domain`, the collection of all returned offsets should form a continuous sequence.
    /// See the [`RemoteStore`] implementation.
    fn digest_offset(&mut self, domain: &str, storage: &Storage) -> Result<usize, crate::Error>;
    /// The async version of `digest_offset`.
    fn digest_offset_async(
        &mut self,
        domain: &str,
        storage: &Storage,
    ) -> impl std::future::Future<Output = Result<usize, crate::Error>> + Send;
}

pub(crate) type BridgeResult<B> = std::result::Result<B, std::io::Error>;

/// Data persistence interface used by [`RemoteStore`].
/// At least one pair of methods should be implemented: `get`+`put` or `get_async`+`put_async`.
/// See examples/remote_store_ureq.rs for a simple implementation to start with.
pub trait ConnectionBridge {
    /// Fetch the storage blob associated with `key`.
    fn get(&self, key: &str) -> BridgeResult<Option<Bytes>>;
    /// Update or insert the storage blob associated with `key` to `body`.
    fn put(&self, key: &str, body: Bytes) -> BridgeResult<()>;
    /// The async version of `get`.
    fn get_async(&self, key: &str) -> impl Future<Output = BridgeResult<Option<Bytes>>> + Send;
    /// The async version of `put`.
    fn put_async(&self, key: &str, body: Bytes) -> impl Future<Output = BridgeResult<()>> + Send;
}

/// Implements [`StorageState`] using binary search to find digests within storage blobs.
/// Retrieved storage blobs are assumed to contain lines of *sorted* digests.
/// Each digest is postfixed with a space-padded offset followed by '\n'.
/// Each line is 68 bytes.
/// example: "9e3b2749dcca704cad379adf3c6894a59c3363f2d78a4a5155555781e69cc     9\n"
#[derive(Debug)]
pub struct RemoteStore<B: ConnectionBridge> {
    #[allow(missing_docs)]
    pub bridge: B,
}

impl<B> StorageState for RemoteStore<B>
where
    B: ConnectionBridge + Send,
{
    #[async_generic]
    #[allow(unused_assignments)]
    fn digest_offset(
        &mut self,
        _domain: &str,
        storage: &Storage,
    ) -> std::result::Result<usize, crate::Error> {
        let key = storage.key.as_str();
        let digest = storage.digest.as_str();

        let mut stored_bytes: Option<Bytes> = None;
        if _async {
            stored_bytes = self.bridge.get_async(key).await?;
        } else {
            stored_bytes = self.bridge.get(key)?;
        }

        // "<digest> <offset>"
        let mut lines: Vec<String> = match stored_bytes {
            None => Vec::default(),
            Some(stored_bytes) => stored_bytes.lines().map_while(|l| l.ok()).collect(),
        };
        // "<digest>"
        let search_lines: Vec<&str> = lines.iter().map(|s| &s[..digest.len()]).collect();

        match search_lines.binary_search(&digest) {
            // return <offset>
            Ok(found_at) => {
                let found_line = &lines[found_at];
                let found_offset: usize = found_line[(digest.len() + 1)..].trim().parse().unwrap();
                Ok(found_offset)
            }
            Err(insert_at) => {
                let next_offset = lines.len();

                // each line is expected to be 68 bytes, to enable HTTP range requests
                lines.insert(insert_at, format!("{digest} {next_offset:>5}"));
                let mut resource = lines.join("\n");
                resource.push('\n');
                let resource_bytes = Bytes::from(resource);

                let mut update_result: Result<(), std::io::Error> = Ok(());
                if _async {
                    update_result = self.bridge.put_async(key, resource_bytes).await;
                } else {
                    update_result = self.bridge.put(key, resource_bytes);
                }

                update_result.map(|_| next_offset).map_err(|e| e.into())
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    /*
    cargo run -F codegen
    cargo test storage -- --nocapture
    */

    use async_generic::async_generic;

    use super::*;
    use crate::identity::{Identity, Population, tests::*};
    use crate::{Error, STORAGE_DIGEST_LENGTH};

    #[tokio::test]
    async fn test_remote_store_async() -> Result<(), Error> {
        impl_test_remote_store_async().await?;
        Ok(())
    }

    #[test]
    fn test_remote_store_blocking() -> Result<(), Error> {
        impl_test_remote_store()?;
        Ok(())
    }

    #[async_generic]
    #[allow(unused_assignments)]
    fn impl_test_remote_store() -> Result<(), Error> {
        let brazilian = Population {
            domain: "br",
            secret: b"0123456789abcdef0123456789abcdef",
            ingredients: &PERFUME_INGREDIENTS,
        };
        let mut store = RemoteStore {
            bridge: MockBridge::default(),
        };

        let mut user1 = Identity::default();
        let mut first_offset = usize::MAX;
        if _async {
            user1 = brazilian.identity_async("f@r.br", &mut store).await?;
            first_offset = store.digest_offset_async("br", &user1.storage).await?;
        } else {
            user1 = brazilian.identity("f@r.br", &mut store)?;
            first_offset = store.digest_offset("br", &user1.storage)?;
        }
        // user1 was assigned to first offset
        assert_eq!(first_offset, 0);

        // subsequent assignments to the same storage blob produce a sequence of offsets
        for i in 1..10 {
            if _async {
                assert_eq!(
                    next_stored_offset_async(&user1.storage, &mut store).await?,
                    i
                );
            } else {
                assert_eq!(next_stored_offset(&user1.storage, &mut store)?, i);
            }
        }

        let storage_object_key = user1.storage.key.as_str();
        let storage_object_contents = store
            .bridge
            .get(storage_object_key)
            .map(|o| o.map(|b| String::from_utf8_lossy(&b[..]).to_string()))
            .unwrap()
            .unwrap();
        let storage_objects = storage_object_contents
            .trim_end()
            .split('\n')
            .collect::<Vec<_>>();
        assert_eq!(storage_objects.len(), 10);
        assert!(storage_objects.iter().all(|o| o.len() == 67));
        println!("contents of {storage_object_key}:\n{storage_object_contents}");

        Ok(())
    }

    #[async_generic]
    #[allow(unused_assignments)]
    fn next_stored_offset(
        init_storage: &Storage,
        store: &mut impl StorageState,
    ) -> Result<usize, Error> {
        // digest offset is incremented when the next digest is assigned to a key
        let mut next_digest_storage = init_storage.clone();
        // same key as user1, but different digest
        next_digest_storage.digest = random_hex_string::<STORAGE_DIGEST_LENGTH>();

        let mut next_offset = usize::MAX;
        if _async {
            next_offset = store
                .digest_offset_async("bt", &next_digest_storage)
                .await?;
        } else {
            next_offset = store.digest_offset("bt", &next_digest_storage)?;
        }

        Ok(next_offset)
    }
}
