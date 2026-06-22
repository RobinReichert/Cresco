---
title: Stepper
layout: default
parent: Modules
nav_order: 7
---

# Stepper

The `stepper` module drives stepper motors. It defines a hardware-agnostic
`Stepper` trait and ships one implementation, `Drv8833`, that drives a bipolar
stepper through a [DRV8833](https://www.ti.com/product/DRV8833) dual H-bridge.
The trait lets the rest of the system command motion without knowing which
driver chip is wired up.

---

## The `Stepper` Trait

`Stepper` is the abstraction every motor driver implements:

| Method | Meaning |
| --- | --- |
| `move_steps(steps)` | Rotate by `steps`. The **sign selects direction**. |
| `hold()` | Energise the coils so the motor resists movement. |
| `release()` | De-energise the coils so the shaft turns freely. |
| `state()` | The current [resting state](#state). |
| `resolution()` | Steps per revolution for the attached motor. |

`move_steps`, `hold` and `release` are `async` and fallible
(`Result<(), Self::Error>`). They are async so that drivers talking to a chip
over a bus (e.g. a UART-controlled driver) can `await` their I/O; a pure-GPIO
driver like `Drv8833` simply returns immediately.

{: .note}
> There is deliberately **no `stop` method**. While `move_steps` is running it
> holds the only `&mut` borrow of the driver, so nothing else can call a method
> on it anyway. To abort a move, drop the `move_steps` future — e.g. with
> `embassy_futures::select` against a cancel condition. The coils are left
> energised at the last step, i.e. holding.

---

## State

A stepper at rest is in exactly one of two physical conditions, so
`StepperState` has two variants:

| State | Coils | Behaviour |
| --- | --- | --- |
| `Holding` | energised | Resists movement (holding torque). |
| `Idle` | de-energised | Shaft turns freely; no current, no heat. |

There is no `Moving` state: while a move is in progress the driver is
exclusively borrowed, so no caller can ever observe it.

---

## `Drv8833`

The DRV8833 is a **dual H-bridge**, not a dedicated stepper controller. It
exposes four logic inputs — `AIN1`, `AIN2`, `BIN1`, `BIN2` — and the driver
generates the phase sequence in firmware. It is constructed from four
configured output pins plus the motor's resolution:

```rust
let motor = Drv8833::new(ain1, ain2, bin1, bin2, /* resolution */ 200);
```

Each pin is an `esp_hal::gpio::Output` configured push-pull and started `Low`,
so the motor boots de-energised:

```rust
let pin = Output::new(peripherals.GPIO0, Level::Low, OutputConfig::default());
```

### Wiring

Four GPIOs drive the DRV8833's logic inputs; the two H-bridge outputs each
drive one motor coil. The motor supply feeds `VM`, and all grounds — ESP32,
DRV8833 and motor supply — must be tied together.

![Image]({{site.baseurl}}/assets/drawio/stepper_wiring.svg)

| ESP32-C3 | DRV8833 in | DRV8833 out | Stepper |
| --- | --- | --- | --- |
| `GPIO0` | `AIN1` | `AOUT1` | Coil A |
| `GPIO1` | `AIN2` | `AOUT2` | Coil A |
| `GPIO2` | `BIN1` | `BOUT1` | Coil B |
| `GPIO3` | `BIN2` | `BOUT2` | Coil B |
| `GND` | `GND` | | |

{: .warning}
> Group the four motor leads into the two coils correctly — both leads of one
> coil on the `A` channel, the other coil on the `B` channel. Use a multimeter:
> the two leads with continuity between them are one coil. Splitting a coil
> across channels makes the motor vibrate without rotating.

### Step Sequence

Driving uses a four-step **wave drive** — exactly one coil is energised at a
time. Each `Step` is a 4-bit pattern (`AIN1 AIN2 BIN1 BIN2`, MSB first):

| Step | Pattern | Energised |
| --- | --- | --- |
| `First` | `0b1000` | Coil A, forward |
| `Second` | `0b0010` | Coil B, forward |
| `Third` | `0b0100` | Coil A, reverse |
| `Fourth` | `0b0001` | Coil B, reverse |

`set_state` shifts the pattern out onto the four pins. `move_steps` advances
one step per iteration (`next` forward, `previous` backward) and waits a fixed
interval between steps.

{: .note}
> Wave drive energises a single coil per step, so the motor draws roughly **half
> the current** of a two-phase-on full-step drive and runs noticeably cooler, at
> the cost of about 30 % less torque.
