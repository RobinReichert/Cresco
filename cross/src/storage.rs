use core::mem::size_of;
use embedded_storage_async::nor_flash::NorFlash;
use logic::{calibration, wifi::LoginData};
use sequential_storage::{
    cache::KeyCacheImpl,
    map::{MapConfig, MapStorage},
};

use crate::partitions;

pub type KeyType = u32;

const CREDENTIALS_KEY: KeyType = 0;
const MAX_PASSWORD_LEN: usize = 64;
const MAX_SSID_LEN: usize = 32;
const HEADER_BYTES: usize = 1;
const CREDENTIALS_BUFFER_SIZE: usize = next_multiple_of(
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
        let config = MapConfig::new(partitions::PARTITIONS.credentials);
        CredentialStorage {
            storage: MapStorage::new(flash, config, cache),
        }
    }

    pub async fn set_credentials(&mut self, credentials: LoginData) -> bool {
        let mut data_buffer = [0u8; CREDENTIALS_BUFFER_SIZE];
        self.storage
            .store_item(&mut data_buffer, &CREDENTIALS_KEY, &credentials)
            .await
            .is_ok()
    }

    pub async fn get_credentials(&mut self) -> Result<LoginData, ()> {
        let mut data_buffer = [0u8; CREDENTIALS_BUFFER_SIZE];
        let credentials = self
            .storage
            .fetch_item(&mut data_buffer, &CREDENTIALS_KEY)
            .await
            .map_err(|_| ())?
            .ok_or(())?;
        return Ok(credentials);
    }
}

const PH_CALIBRATION_KEY: KeyType = 0;
const EC_CALIBRATION_KEY: KeyType = 1;
const CALIBRATION_BUFFER_SIZE: usize = next_multiple_of(16 + size_of::<KeyType>(), 1);

pub struct CalibrationStorage<F, C>
where
    F: NorFlash,
    C: KeyCacheImpl<KeyType>,
{
    storage: MapStorage<KeyType, F, C>,
}

impl<F, C> CalibrationStorage<F, C>
where
    F: NorFlash,
    C: KeyCacheImpl<KeyType>,
{
    pub fn new(flash: F, cache: C) -> Self {
        let config = MapConfig::new(partitions::PARTITIONS.calibration);
        CalibrationStorage {
            storage: MapStorage::new(flash, config, cache),
        }
    }

    pub async fn set_ph_calibration(&mut self, calibration: calibration::Linear) -> bool {
        let mut data_buffer = [0u8; CALIBRATION_BUFFER_SIZE];
        self.storage
            .store_item(&mut data_buffer, &PH_CALIBRATION_KEY, &calibration)
            .await
            .is_ok()
    }

    pub async fn get_ph_calibration(&mut self) -> Result<calibration::Linear, ()> {
        let mut data_buffer = [0u8; CALIBRATION_BUFFER_SIZE];
        let calibration = self
            .storage
            .fetch_item(&mut data_buffer, &PH_CALIBRATION_KEY)
            .await
            .map_err(|_| ())?
            .ok_or(())?;
        return Ok(calibration);
    }

    pub async fn set_ec_calibration(&mut self, calibration: calibration::Linear) -> bool {
        let mut data_buffer = [0u8; CALIBRATION_BUFFER_SIZE];
        self.storage
            .store_item(&mut data_buffer, &EC_CALIBRATION_KEY, &calibration)
            .await
            .is_ok()
    }

    pub async fn get_ec_calibration(&mut self) -> Result<calibration::Linear, ()> {
        let mut data_buffer = [0u8; CALIBRATION_BUFFER_SIZE];
        let calibration = self
            .storage
            .fetch_item(&mut data_buffer, &EC_CALIBRATION_KEY)
            .await
            .map_err(|_| ())?
            .ok_or(())?;
        return Ok(calibration);
    }
}
