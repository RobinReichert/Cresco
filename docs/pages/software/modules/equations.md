---
title: Equations
layout: default
parent: Modules
nav_order: 9
---

# Equations

The `equations` module holds the mathematical functions the firmware uses. It
currently provides `Linear`, a linear function `y = m*x + t`. It is used to
calibrate sensors, translating a raw reading into a calibrated value (e.g.
converting a raw pH probe voltage into an actual pH value).

## Usage

A `Linear` is created from its slope `m` and intercept `t`, or fitted through
two `Point { x, y }`:

```rust
let linear = Linear::new(m, t);
let linear = Linear::from_points(first, second)?;

let y = linear.apply(x);
```

`from_points` fails if the points share an `x` (vertical line) or a `y`
(constant line).

`apply` evaluates `m*x + t`. A `Linear` is always valid; a sensor that is not
calibrated yet is represented by the absence of one (`Option<Linear>`).

The measurement module uses `from_points` to fit a sensor's line through the
two calibration points.
