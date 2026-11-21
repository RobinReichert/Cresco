---
title: Blackboard
layout: default
parent: Modules
nav_order: 6
---

# Blackboard

The blackboard is the shared, in-RAM store of plant-health **facts** — the
latest measured values such as pH, EC, temperature and water level. It lets
the many tasks that need this data read it from one place instead of passing
data objects between them.

---

## Crate Split

The blackboard is split across the two crates, the same way as the other
modules:

| Part | Lives in | Contents |
| --- | --- | --- |
| Data type (`Measurements`) | `logic` | a plain `no_std` struct of facts |
| Store+accessors | `cross` | the `static` mutex and the read/write functions |

Keeping the data type in `logic` means it has no dependency on the async
runtime, so it stays host-testable. The shared `static` and its locking
depend on `embassy-sync` and therefore live in `cross`.

---

## Access

The store is a single `Mutex` over the whole struct, accessed through a small
set of functions rather than the raw `static`:

```rust
static MEASUREMENTS: Mutex<CriticalSectionRawMutex, Measurements> =
    Mutex::new(Measurements::DEFAULT);

pub async fn measurements() -> Measurements {
    MEASUREMENTS.lock().await.clone()
}
pub async fn set_ph(ph: Float) { MEASUREMENTS.lock().await.ph = ph }
```

A reader calls `measurements()` to copy out a coherent view of all facts at
once; a writer uses a field setter. Routing every access through these
functions is what keeps the locking rules below in one place.

---

## Rules

These conventions keep the shared state correct and contention-free:

- **Single writer per field.** Each fact is written by exactly one task (the
  task measuring that sensor) and only read by everyone else. This removes
  write contention by design.
- **Lock, copy, release — never across `.await`.** A critical section only
  copies values out and returns; it must not hold the lock across an `await`,
  which would block every other task.
- **One mutex over the whole struct.** A single lock hands readers a
  *consistent* multi-field snapshot. Per-field locks are unnecessary and would
  tear that consistency.

---

## Targets

Next to the measured facts, the blackboard holds the target equations the user
chose, `Targets { ph, ec }`, in a second `Mutex`. `dose_task` reads them with
`targets()` each period and the web server reads and replaces them with
`targets()` and `set_targets()`. They start as a constant of 5.0 until
the stored targets are loaded at boot. When the web server replaces them, it
also writes them to flash, so they survive a restart. If nothing is stored yet,
the constant stays.
