---
title: Dose
layout: default
parent: Modules
nav_order: 11
---

# Dose

The `dose` module closes the control loop. Once per period it reads the latest
EC and pH from the
[blackboard]({{ site.baseurl }}{% link pages/software/modules/blackboard.md %})
, runs a
[PID controller]({{ site.baseurl }}{% link pages/software/modules/pid.md %})
for each, and drives three
[stepper]({{ site.baseurl }}{% link pages/software/modules/stepper.md %})
pumps to push the readings back towards their targets.

Unlike `wifi` or `measurement` there is no manager/driver split: the whole
module is a single task, `dose_task`, with no host-testable logic of its own.
The control maths lives in `logic::pid`.

---

## Pumps

`dose_task` owns three pumps, all of type `ConcreteStepper`

| Pump | Effect |
| --- | --- |
| `ec_pump` | Adds nutrient solution — raises EC. |
| `ph_up_pump` | Adds pH-up solution — raises pH. |
| `ph_down_pump` | Adds pH-down solution — lowers pH. |

---

## Control Cycle

The task is started from `main` and loops forever:

1. **Wait for the period.** Sleep until `next_period`.
2. **Wait for fresh data.** Block on the `fresh` signal, which the
   measurement task raises after writing a new pH/EC pair to the blackboard.
   The signal carries the measurement time in milliseconds since boot.
3. **Schedule the next cycle** for `now + PERIOD_LENGTH`.
4. **Take a blackboard snapshot.** If either EC or pH is still `None`, nothing
   happens this cycle.
5. **Dose** as described below.

The next period is scheduled *after* the fresh signal arrives, so a late
measurement shifts the whole cycle rather than leaving less than
`PERIOD_LENGTH` between two doses.

### Dosing Decision

EC is corrected first and **takes priority over pH**, because adding nutrient
solution shifts the pH:

```text
ec_steps = ec_pid.calculate(ec, target, period) * RATE
if ec_steps > THRESHOLD:
    move ec_pump by ec_steps, release it
    ph_pid.reset()            # pH is now out of date
else:
    ph_steps = ph_pid.calculate(ph, target, period) * RATE
    if |ph_steps| > THRESHOLD:
        positive -> ph_up_pump   moves  |ph_steps|
        negative -> ph_down_pump moves  |ph_steps|
        release the pump
```

After each dose the pump is `release`d so the coils are de-energised and the
motor stays cool between cycles.

| Constant | Value | Meaning |
| --- | --- | --- |
| `PERIOD_LENGTH` | 60 s | Minimum time between two dosing cycles. |
| `THRESHOLD` | `1.0` | Smallest scaled control signal (`PID output * RATE`) |
| `RATE` | `100.0` | Steps per unit of controller output. |

Both PID controllers use `Kp = 0.8`, `Ki = 0.1`, `Kd = 1.0`.
