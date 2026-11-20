use core::{fmt::Debug, mem::size_of};
use defmt::{Debug2Format, debug, warn};
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

fn log_store<E: Debug>(what: &str, result: Result<(), E>) -> bool {
    match result {
        Ok(()) => {
            debug!("storage: stored {}", what);
            true
        }
        Err(e) => {
            warn!("storage: store {} failed: {}", what, Debug2Format(&e));
            false
        }
    }
}

fn log_fetch<T, E: Debug>(what: &str, result: Result<Option<T>, E>) -> Result<T, ()> {
    match result {
        Ok(Some(item)) => {
            debug!("storage: fetched {}", what);
            Ok(item)
        }
        Ok(None) => {
            debug!("storage: {} not found", what);
            Err(())
        }
        Err(e) => {
            warn!("storage: fetch {} failed: {}", what, Debug2Format(&e));
            Err(())
        }
    }
}

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
        let result = self
            .storage
            .lock()
            .await
            .store_item(&mut data_buffer, &CREDENTIALS_KEY, &credentials)
            .await;
        log_store("credentials", result)
    }

    pub async fn get_credentials(&self) -> Result<LoginData, ()> {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        let result = self
            .storage
            .lock()
            .await
            .fetch_item(&mut data_buffer, &CREDENTIALS_KEY)
            .await;
        log_fetch("credentials", result)
    }

    pub async fn set_ph_calibration(&self, calibration: equations::Equation) -> bool {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        let result = self
            .storage
            .lock()
            .await
            .store_item(&mut data_buffer, &PH_CALIBRATION_KEY, &calibration)
            .await;
        log_store("ph calibration", result)
    }

    pub async fn get_ph_calibration(&self) -> Result<equations::Equation, ()> {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        let result = self
            .storage
            .lock()
            .await
            .fetch_item(&mut data_buffer, &PH_CALIBRATION_KEY)
            .await;
        log_fetch("ph calibration", result)
    }

    pub async fn set_ec_calibration(&self, calibration: equations::Equation) -> bool {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        let result = self
            .storage
            .lock()
            .await
            .store_item(&mut data_buffer, &EC_CALIBRATION_KEY, &calibration)
            .await;
        log_store("ec calibration", result)
    }

    pub async fn get_ec_calibration(&self) -> Result<equations::Equation, ()> {
        let mut data_buffer = [0u8; BUFFER_SIZE];
        let result = self
            .storage
            .lock()
            .await
            .fetch_item(&mut data_buffer, &EC_CALIBRATION_KEY)
            .await;
        log_fetch("ec calibration", result)
    }
}

pub type ConcreteStorage = Storage<BlockingAsync<FlashStorage<'static>>, NoCache>;
