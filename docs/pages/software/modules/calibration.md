---
title: Calibration
layout: default
parent: Modules
nav_order: 9
---

# Calibration

The `calibration` module provides `Linear`, a two-point linear calibration
for translating a raw sensor reading into a calibrated value (e.g. converting
a raw pH probe voltage into an actual pH value).

## Usage

A `Linear` is created empty and calibrated with two `Point { x, y }` readings,
where `x` is the raw input and `y` is the known reference value:

```rust
let mut linear = Linear::new();
linear.calibrate(
    Point { x: raw_low, y: reference_low },
    Point { x: raw_high, y: reference_high },
)?;

let calibrated = linear.apply(raw_reading)?;
```

`calibrate` fits the line `y = m*x + t` through the two points. `apply` then
maps any raw `x` through that line.
