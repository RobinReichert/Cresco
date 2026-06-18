---
title: Pid
layout: default
parent: Modules
nav_order: 5
---

# Pid

The `pid` module implements a **PID controller** used to keep a measured
process value at a desired target. It drives pH and EC correction for the
system: given a target pH and the current reading from the probe,
it computes how much corrective action (dosing) should be applied.

## Controller

The controller is constructed with the three gains and is then driven by
repeated calls to `calculate`, one per control cycle.

| Parameter | Meaning |
| --- | --- |
| `current` | The latest measured value (e.g. current pH). |
| `target` | The setpoint the controller drives towards. |
| `millis` | A monotonic timestamp in **milliseconds**. |

The return value is the control signal — the corrective action to apply, for
example the dosing amount or pump duty. Its sign indicates direction: a
positive output means the measured value is below the target and must be
raised, a negative output means it is above and must be lowered.

## The Three Terms

On each call the controller computes the `error = target - current` and
combines three terms:

- **Proportional** (`kp * error`) — reacts to the present error. Larger error,
  larger correction. Acts immediately but alone never fully settles on the
  target.
- **Integral** (`ki * integral`) — accumulates `error * dt` over time, so small
  persistent offsets are eventually corrected.
- **Derivative** (`kd * (error - last_error) / dt`) — reacts to how fast the
  error is changing and dampens the approach, reducing overshoot.

The sum of the three terms is the control signal.

## Timing

`dt` is derived from the difference between successive `millis` values, so the
integral and derivative are correctly scaled by the real interval between
control cycles. This keeps the tuning independent of how often the loop runs.

{: .note}
> Because `dt` is measured in **milliseconds**, the `ki` and `kd` gains are
> scaled relative to a seconds-based time base. Expect `ki` in particular to be
> a small value once tuned for a real system.

{: .warning}
> The first `calculate` call only seeds the internal state and returns the
> proportional term — it does not feed the integral. This avoids a startup
> spike from the initially unknown `dt`. If two calls share the same `millis`
> (`dt == 0`), the derivative term is skipped to avoid a division by zero.
