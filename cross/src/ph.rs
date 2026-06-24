use logic::Float;

#[allow(async_fn_in_trait)]
pub trait PhProbe {
    type Error;
    async fn read(&mut self) -> Result<Float, Self::Error>;
}

pub fn correct_for_temperature(ph: Float, _temp: Float) -> Float {
    ph
}

pub mod analog {

    use super::*;
    use esp_hal::{
        Async,
        analog::adc::{Adc, AdcCalCurve, AdcChannel, AdcConfig, AdcPin},
        gpio::AnalogPin,
        peripherals::ADC1,
    };

    pub struct AnalogPhProbe<'a, PIN: AdcChannel> {
        adc: Adc<'a, ADC1<'a>, Async>,
        pin: AdcPin<PIN, ADC1<'a>, AdcCalCurve<ADC1<'a>>>,
    }

    impl<'a, PIN: AdcChannel + AnalogPin> AnalogPhProbe<'a, PIN> {
        pub fn new(adc1: ADC1<'a>, input: PIN) -> Self {
            let mut config = AdcConfig::new();
            let pin = config.enable_pin_with_cal::<PIN, AdcCalCurve<ADC1<'a>>>(
                input,
                esp_hal::analog::adc::Attenuation::_11dB,
            );
            Self {
                adc: Adc::new(adc1, config).into_async(),
                pin,
            }
        }
    }

    impl<'a, PIN: AdcChannel + AnalogPin> PhProbe for AnalogPhProbe<'a, PIN> {
        type Error = ();

        async fn read(&mut self) -> Result<Float, Self::Error> {
            Ok(self.adc.read_oneshot(&mut self.pin).await as Float)
        }
    }
}
