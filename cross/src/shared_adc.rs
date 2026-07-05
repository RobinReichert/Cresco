use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use esp_hal::{
    Async,
    analog::adc::{Adc, AdcCalCurve, AdcChannel, AdcConfig, AdcPin},
    gpio::AnalogPin,
    peripherals::ADC1,
};

pub type ProbePin<'a, PIN> = AdcPin<PIN, ADC1<'a>, AdcCalCurve<ADC1<'a>>>;

pub struct AdcBuilder<'a> {
    config: AdcConfig<ADC1<'a>>,
}

impl<'a> AdcBuilder<'a> {
    pub fn new() -> Self {
        Self {
            config: AdcConfig::new(),
        }
    }

    pub fn add_pin<PIN: AdcChannel + AnalogPin>(&mut self, pin: PIN) -> ProbePin<'a, PIN> {
        self.config
            .enable_pin_with_cal::<PIN, AdcCalCurve<ADC1<'a>>>(
                pin,
                esp_hal::analog::adc::Attenuation::_11dB,
            )
    }

    pub fn build(self, adc1: ADC1<'a>) -> SharedAdc<'a> {
        SharedAdc {
            adc: Mutex::new(Adc::new(adc1, self.config).into_async()),
        }
    }
}

pub struct SharedAdc<'a> {
    adc: Mutex<CriticalSectionRawMutex, Adc<'a, ADC1<'a>, Async>>,
}

pub struct AdcInterface<'a> {
    shared_adc: &'a SharedAdc<'a>,
}

impl<'a> AdcInterface<'a> {
    pub fn new(shared_adc: &'a SharedAdc<'a>) -> Self {
        Self { shared_adc }
    }

    pub async fn read_oneshot<PIN: AdcChannel>(&mut self, pin: &mut ProbePin<'a, PIN>) -> u16 {
        let mut adc = self.shared_adc.adc.lock().await;
        adc.read_oneshot(pin).await
    }
}
