---
title: Ph
layout: default
parent: Modules
nav_order: 8
---

# Ph

The `ph` module reads a pH probe. It defines a hardware-agnostic `PhProbe`
trait and ships one implementation, `AnalogPhProbe`, that samples a probe's
analog output through the ESP32-C3's ADC. The trait lets the rest of the system
read pH without knowing how the probe is wired up.

---

## The `PhProbe` Trait

`PhProbe` is the abstraction every probe implements:

| Method | Meaning |
| --- | --- |
| `read()` | Take a reading. `async`, fallible (`Result<Float, Self::Error>`). |

`read` is `async` so that probes talking to a chip over a bus can `await` their
I/O; the analog implementation drives the ADC and resolves once the conversion
completes.

---

## `AnalogPhProbe`

The analog probe is built from the `ADC1` peripheral and the GPIO carrying the
probe's analog output:

```rust
let mut probe = AnalogPhProbe::new(peripherals.ADC1, peripherals.GPIO0);
```

On the ESP32-C3, `GPIO0` is `ADC1` channel 0. The pin is enabled at **11 dB
attenuation** so the ADC covers the widest input range the chip offers.

### Calibration

The pin is enabled with the **`AdcCalCurve`** calibration scheme:

```rust
config.enable_pin_with_cal::<PIN, AdcCalCurve<ADC1>>(input, Attenuation::_11dB);
```

This is not optional polish — it is required for correct readings. The
calibration scheme sets the ADC's zero-bias init code from the chip's eFuse and
returns the result in **millivolts** via curve fitting.

### Sampling

`read` does not return a single conversion. It takes **16 samples**, discards
the single highest and lowest, and averages the remaining 14:

```rust
let avg = (sum - min - max) / (SAMPLES - 2);
```

Averaging cuts the ADC's random noise by roughly `√N`, and dropping the
extremes rejects the occasional outlier spike — most visible while Wi-Fi is
transmitting, which couples noise into the ADC. pH changes slowly, so the cost
of 16 conversions per reading is irrelevant.
