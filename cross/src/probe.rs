use esp_hal::analog::adc::AdcChannel;
use logic::Float;

use crate::shared_adc::{AdcInterface, ProbePin};

pub struct AnalogProbe<'a, PIN: AdcChannel> {
    adc: AdcInterface<'a>,
    pin: ProbePin<'a, PIN>,
}

impl<'a, PIN: AdcChannel> AnalogProbe<'a, PIN> {
    pub fn new(adc: AdcInterface<'a>, pin: ProbePin<'a, PIN>) -> Self {
        Self { adc, pin }
    }

    pub async fn read(&mut self) -> Result<Float, ()> {
        const SAMPLES: u32 = 16;
        const VOLTAGE_DIVISOR: u32 = 2;
        let mut sum: u32 = 0;
        let mut min = u16::MAX;
        let mut max = 0;
        for _ in 0..SAMPLES {
            let v = self.adc.read_oneshot(&mut self.pin).await;
            sum += v as u32;
            min = min.min(v);
            max = max.max(v);
        }
        let avg = (sum - min as u32 - max as u32) / (SAMPLES - 2); //drop min and max to counteract spikes
        Ok((avg * VOLTAGE_DIVISOR) as Float) //times two because of voltage divider
    }
}
