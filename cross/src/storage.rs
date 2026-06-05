use core::mem::size_of;
use embedded_storage_async::nor_flash::NorFlash;
use logic::wifi::LoginData;
use sequential_storage::{
    cache::KeyCacheImpl,
    map::{MapConfig, MapStorage},
};

use crate::partitions;

pub type KeyType = u32;

const PASSWORD_KEY: KeyType = 0;
const MAX_PASSWORD_LEN: usize = 64;
const MAX_SSID_LEN: usize = 32;
const HEADER_BYTES: usize = 1;
const BUFFER_SIZE: usize = next_multiple_of(
    HEADER_BYTES + MAX_PASSWORD_LEN + MAX_SSID_LEN + size_of::<KeyType>(),
    1,
);

const fn next_multiple_of(n: usize, multiple: usize) -> usize {
    (n + multiple - 1) / multiple * multiple
}

pub struct CredentialStorage<F, C>
where
    F: NorFlash,
    C: KeyCacheImpl<KeyType>,
{
    storage: MapStorage<KeyType, F, C>,
}

impl<F, C> CredentialStorage<F, C>
where
    F: NorFlash,
    C: KeyCacheImpl<KeyType>,
{
    pub fn new(flash: F, cache: C) -> Self {
        let config = MapConfig::new(partitions::PARTITIONS.storage);
        CredentialStorage {
            storage: MapStorage::new(flash, config, cache),
        }
    }

    pub async fn set_credentials(&mut self, credentials: LoginData) -> bool {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        self.storage
            .store_item(&mut data_buffer, &PASSWORD_KEY, &credentials)
            .await
            .is_ok()
    }

    pub async fn get_credentials(&mut self) -> Result<LoginData, ()> {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        let credentials = self
            .storage
            .fetch_item(&mut data_buffer, &PASSWORD_KEY)
            .await
            .map_err(|_| ())?
            .ok_or(())?;
        return Ok(credentials);
    }
}
