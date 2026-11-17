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
| `FinishingEc` | EC calibration computed, waiting for the user to finish it |
| `FinishingPh` | pH calibration computed, waiting for the user to finish it |

### `MeasurementEvent`

| Variant | When to fire |
| --- | --- |
| `Start` | Once, at boot; measures immediately if both sensors are calibrated |
| `StartMeasurement` | The periodic timer elapsed |
| `FinishCalibration` | The user finished the calibration; apply and store it |
| `CalibrationStored` | A calibration was stored; measure if both are set |
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
| `WaitForFinish` | Calibration computed; wait for the user to finish or abort |
| `Store(Ec/Ph)Calibration { calibration }` | Calibration finished; store it |
| `WriteMeasurements { ec, ph }` | Write calibrated readings to the blackboard |
| `ShowError { error }` | Report a `MeasurementError` |
| `WaitForNext` | Nothing to do; block until the next trigger |

## Calibration

Each sensor owns its own `Option<equations::Linear>`, calibrated
independently; `None` means not calibrated yet. A measurement cycle only runs
once **both** are `Some` — `Idle` + `StartMeasurement` checks this before
proceeding to `MeasuringEc`; until then it returns
`ShowError { error: NotCalibratedYet }` and stays in `Idle`.

Measuring does not wait for the periodic timer in two cases: at boot, when
both calibrations were loaded from storage (`Start`), and right after a
calibration is stored (`CalibrationStored`), provided the other sensor is
already calibrated too.

A calibration is not applied when the second point is taken. The manager
computes it into `FinishingEc`/`FinishingPh` and waits for `FinishCalibration`,
which only then replaces the sensor's calibration and stores it. `Abort` in
these states discards the computed calibration and keeps the old one. If the
two points cannot be fitted, the manager reports `CalibrationFailed` and
returns to `Idle` without entering the finishing state.

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

`CalibrationAborted` is set when something other than the expected calibration
point (or for `FinishCalibration`) arrives while the driver waits for one
(e.g. an abort).
