---
title: Measurement
layout: default
parent: Modules
nav_order: 10
---

# Measurement

The `measurement` module runs the periodic pH/EC measurement cycle and the
two-point calibration flow for each sensor. Like `wifi` and `dhcp`, it
separates the *manager* — a state machine deciding what should happen — from
the *driver* that carries it out.

---

## State Machine

### State Diagram

![Image]({{site.baseurl}}/assets/drawio/measurement.svg)

### States

| State | Meaning |
| --- | --- |
| `Start` | Initial state, before the first `Start` event |
| `Idle` | Waiting for the next periodic tick or a calibration command |
| `MeasuringEc` | Reading the EC probe |
| `MeasuringPh` | Reading the pH probe (carries the EC value just read) |
| `CalibratingEcFirst` | Waiting for the first EC calibration point |
| `CalibratingEcSecond` | Waiting for the second EC calibration point |
| `CalibratingPhFirst` | Waiting for the first pH calibration point |
| `CalibratingPhSecond` | Waiting for the second pH calibration point |

### `MeasurementEvent`

| Variant | When to fire |
| --- | --- |
| `Start` | Once, at boot |
| `StartMeasurement` | The periodic timer elapsed |
| `Start(Ec/Ph)Calibration` | User requested a calibration |
| `FirstMeasured` / `SecondMeasured` | Probe was read for a calibration point |
| `(Ec/Ph)Measured` | Routine measurement of that sensor completed |
| `Abort` | Cancel whatever is in progress |

### `MeasurementAction`

| Variant | Meaning |
| --- | --- |
| `Ignore` | The event was not valid in the current state; do nothing |
| `MeasureEc` / `MeasurePh` | Read that probe |
| `Retrieve(Ec/Ph)(First/Second)` | Read probe at Calibration point |
| `WriteMeasurements { ec, ph }` | Write calibrated readings to the blackboard |
| `ShowError { error }` | Report a `MeasurementError` |
| `WaitForNext` | Nothing to do; block until the next trigger |

## Calibration

Each sensor owns its own `calibration::Linear`, calibrated independently. A
measurement cycle only runs once **both** are calibrated — `Idle` +
`StartMeasurement` checks `is_calibrated()` on each before proceeding to
`MeasuringEc`; until then it returns `ShowError { error: NotCalibratedYet }`
and stays in `Idle`.

`FirstMeasured`/`SecondMeasured` carry both the raw probe reading and the
actual reference value the user provided (e.g. the pH of a buffer solution),
so each sensor's calibration line is fit from real user input rather than
fixed constants.

---

## Driver

`measurement_task` runs the loop that feeds events into the manager and
carries out the actions it returns — reading probes, writing the blackboard,
and blocking on a timer or a command between cycles. Two types cross the
driver/web boundary:

| Type | Direction |
| --- | --- |
| `MeasurementCommand` | web → driver |
| `MeasurementStatus` | driver → web |

`MeasurementStatus` is held in a `Mutex`, not a channel: every update
unconditionally replaces the previous value, so a reader always sees the
current status rather than draining a queue of stale ones.
