use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_storage_async::nor_flash::{ErrorType, NorFlash, ReadNorFlash};

pub struct SharedFlash<F>
where
    F: NorFlash,
{
    flash: Mutex<NoopRawMutex, F>,
    capacity: usize,
}

impl<F> SharedFlash<F>
where
    F: NorFlash,
{
    pub fn new(flash: F) -> Self {
        let capacity = flash.capacity();
        Self {
            flash: Mutex::new(flash),
            capacity,
        }
    }
}

pub struct SharedFlashInterface<'a, F>
where
    F: NorFlash,
{
    shared_flash: &'a SharedFlash<F>,
}

impl<'a, F> SharedFlashInterface<'a, F>
where
    F: NorFlash,
{
    pub fn new(shared_flash: &'a SharedFlash<F>) -> Self {
        Self { shared_flash }
    }
}

impl<'a, F> ErrorType for SharedFlashInterface<'a, F>
where
    F: NorFlash,
{
    type Error = F::Error;
}

impl<'a, F> ReadNorFlash for SharedFlashInterface<'a, F>
where
    F: NorFlash,
{
    const READ_SIZE: usize = F::READ_SIZE;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let mut f = self.shared_flash.flash.lock().await;
        f.read(offset, bytes).await
    }

    fn capacity(&self) -> usize {
        self.shared_flash.capacity
    }
}

impl<'a, F> NorFlash for SharedFlashInterface<'a, F>
where
    F: NorFlash,
{
    const WRITE_SIZE: usize = F::WRITE_SIZE;
    const ERASE_SIZE: usize = F::ERASE_SIZE;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let mut f = self.shared_flash.flash.lock().await;
        f.erase(from, to).await
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let mut f = self.shared_flash.flash.lock().await;
        f.write(offset, bytes).await
    }
}
