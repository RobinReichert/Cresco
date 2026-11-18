---
title: Equations
layout: default
parent: Modules
nav_order: 9
---

# Equations

The `equations` module holds the mathematical functions the firmware uses as
one enum, `Equation`. It currently has a single variant:

| Variant | Function |
| --- | --- |
| `Linear { m, t }` | `y = m*x + t` |

`Equation::Linear` is used to calibrate sensors, translating a raw reading
into a calibrated value (e.g. converting a raw pH probe voltage into an actual
pH value).

## Usage

A linear equation is built from its slope `m` and intercept `t`, or fitted
through two `Point { x, y }`. `y` evaluates the equation at `x`:

```rust
let linear = Equation::Linear { m, t };
let linear = Equation::linear_from_points(first, second)?;

let y = linear.y(x)?;
```

`linear_from_points` fails if the points share an `x` (vertical line) or a `y`
(constant line). `y` returns a `Result`, so an equation that cannot be
evaluated can report it. A sensor that is not calibrated yet is represented
by the absence of an equation (`Option<Equation>`).

The measurement module uses `linear_from_points` to fit a sensor's line
through the two calibration points.

## Storage

`Equation` implements `sequential_storage::map::Value`. It is stored as one
tag byte naming the variant (`1` = `Linear`), followed by the variant's
values as little-endian floats (`m`, then `t`). Data without a known tag is
rejected as invalid, so a calibration stored before the tag was introduced is
treated as missing and the sensor has to be calibrated again.
