//! Persistent random name generator.

mod population;
mod storage;

pub use population::{Ingredients, Population};
pub use storage::{ConnectionBridge, RemoteStore, Storage, StorageState};

/// A distinct value generated from a population.
#[derive(Debug)]
pub struct Identity<'dom> {
    /// Shared by all members of a population.
    pub domain: &'dom str,
    /// Unique to this member.
    pub friendly_name: String,
    /// Needed to ensure that an identifier always maps to the same name.
    /// See [`StorageState`].
    pub storage: storage::Storage,
}

impl<'dom> PartialEq for Identity<'dom> {
    fn eq(&self, other: &Self) -> bool {
        self.domain == other.domain && self.friendly_name == other.friendly_name
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::RwLock;

    use async_generic::async_generic;
    use bytes::Bytes;

    use super::*;
    use crate::hex_string::HexString;
    use crate::identity::storage::BridgeResult;

    include!(concat!(env!("TMP_DIR"), "/perfume.rs"));

    #[derive(Default)]
    pub struct MockBridge {
        resources: RwLock<HashMap<String, Bytes>>,
    }

    impl ConnectionBridge for MockBridge {
        #[async_generic]
        fn get(&self, key: &str) -> BridgeResult<Option<Bytes>> {
            let resources = self.resources.read().unwrap();
            let bytes = resources.get(key).map(|b| b.to_owned());
            Ok(bytes)
        }
        #[async_generic]
        fn put(&self, key: &str, body: Bytes) -> BridgeResult<()> {
            let mut resources = self.resources.write().unwrap();
            resources.entry(key.to_string()).insert_entry(body);
            Ok(())
        }
    }

    impl<'dom> Default for Identity<'dom> {
        fn default() -> Self {
            Self {
                domain: "",
                friendly_name: String::new(),
                storage: Storage {
                    key: HexString::<3>::default(),
                    digest: HexString::<61>::default(),
                },
            }
        }
    }

    pub fn random_hex_string<const N: usize>() -> HexString<N> {
        use rand::prelude::*;
        let mut rng = rand::rng();
        let random_hex_byte = || match rng.random_range(0..16) as u8 {
            number if number < 10 => number + 0x30,
            alpha => alpha - 10 + 0x61,
        };

        let mut buf = [0; N];
        buf.fill_with(random_hex_byte);
        HexString::from(&buf[..])
    }
}
