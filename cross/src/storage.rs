use core::mem::size_of;
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embedded_storage_async::nor_flash::NorFlash;
use esp_storage::FlashStorage;
use logic::{equations, wifi::LoginData};
use sequential_storage::{
    cache::{KeyCacheImpl, NoCache},
    map::{MapConfig, MapStorage},
};

use crate::partitions;

pub type KeyType = u32;

const CREDENTIALS_KEY: KeyType = 0;
const PH_CALIBRATION_KEY: KeyType = 1;
const EC_CALIBRATION_KEY: KeyType = 2;
const MAX_PASSWORD_LEN: usize = 64;
const MAX_SSID_LEN: usize = 32;
const HEADER_BYTES: usize = 1;
const CREDENTIALS_BUFFER_SIZE: usize = HEADER_BYTES + MAX_PASSWORD_LEN + MAX_SSID_LEN;
const CALIBRATION_BUFFER_SIZE: usize = 16;
const BUFFER_SIZE: usize = size_of::<KeyType>()
    + if CREDENTIALS_BUFFER_SIZE > CALIBRATION_BUFFER_SIZE {
        CREDENTIALS_BUFFER_SIZE
    } else {
        CALIBRATION_BUFFER_SIZE
    };

pub struct Storage<F, C>
where
    F: NorFlash,
    C: KeyCacheImpl<KeyType>,
{
    storage: Mutex<CriticalSectionRawMutex, MapStorage<KeyType, F, C>>,
}

impl<F, C> Storage<F, C>
where
    F: NorFlash,
    C: KeyCacheImpl<KeyType>,
{
    pub fn new(flash: F, cache: C) -> Self {
        let config = MapConfig::new(partitions::PARTITIONS.storage);
        Storage {
            storage: Mutex::new(MapStorage::new(flash, config, cache)),
        }
    }

    pub async fn set_credentials(&self, credentials: LoginData) -> bool {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        self.storage
            .lock()
            .await
            .store_item(&mut data_buffer, &CREDENTIALS_KEY, &credentials)
            .await
            .is_ok()
    }

    pub async fn get_credentials(&self) -> Result<LoginData, ()> {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        let credentials = self
            .storage
            .lock()
            .await
            .fetch_item(&mut data_buffer, &CREDENTIALS_KEY)
            .await
            .map_err(|_| ())?
            .ok_or(())?;
        return Ok(credentials);
    }

    pub async fn set_ph_calibration(&self, calibration: equations::Linear) -> bool {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        self.storage
            .lock()
            .await
            .store_item(&mut data_buffer, &PH_CALIBRATION_KEY, &calibration)
            .await
            .is_ok()
    }

    pub async fn get_ph_calibration(&self) -> Result<equations::Linear, ()> {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        let calibration = self
            .storage
            .lock()
            .await
            .fetch_item(&mut data_buffer, &PH_CALIBRATION_KEY)
            .await
            .map_err(|_| ())?
            .ok_or(())?;
        return Ok(calibration);
    }

    pub async fn set_ec_calibration(&self, calibration: equations::Linear) -> bool {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        self.storage
            .lock()
            .await
            .store_item(&mut data_buffer, &EC_CALIBRATION_KEY, &calibration)
            .await
            .is_ok()
    }

    pub async fn get_ec_calibration(&self) -> Result<equations::Linear, ()> {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        let calibration = self
            .storage
            .lock()
            .await
            .fetch_item(&mut data_buffer, &EC_CALIBRATION_KEY)
            .await
            .map_err(|_| ())?
            .ok_or(())?;
        return Ok(calibration);
    }
}

pub type ConcreteStorage = Storage<BlockingAsync<FlashStorage<'static>>, NoCache>;
