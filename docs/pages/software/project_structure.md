---
title: Project Structure
layout: default
parent: Software
nav_order: 3
---

# Project Structure

## `cross/` — Application Crate

The main embedded application that runs on the ESP32-C3.

```text
cross/
├── .cargo/
│   └── config.toml # Sets default target to riscv32imc-unknown-none-elf
├── build.rs        # Build script (e.g. linker config, code generation)
├── Cargo.toml
├── README.md
└── src/
    ├── bin/
    │   └── main.rs # Binary entry point
    └── lib.rs      # Library root (shared between binary and tests)
```

`main.rs` is the firmware entry point; hardware-independent logic lives in the
`logic` crate.

---

## `logic/` — Platform-Independent Logic Crate

```text
logic/
├── Cargo.toml
├── README.md
└── src/
    └── lib.rs
```

General business logic with no hardware dependencies. All code here should be
pure Rust that compiles on the host — if a module needs to `use esp_hal` or
reference a peripheral type, it belongs in `cross/` instead. Run the test suite
with:

```bash
./cresco.sh test
```

No target flag, no connected hardware, no emulator required.

---

## What belongs where

| Belongs in `logic/` | Belongs in `cross/` |
| --- | --- |
| Business logic and rules | Peripheral and HAL initialisation |
| Data models and types | Embassy tasks and executors |
| Serialisation / deserialisation | Interrupt handlers |
| Pure algorithms and maths | Device-specific configuration |
| Anything you want to unit-test | Firmware entry point (`main.rs`) |

{: .tip}
> If you find yourself reaching for a hardware peripheral inside `logic/`, that
> is a sign the code belongs in `cross/` instead — or that an abstraction
> boundary (a trait) is missing.

---

## `docs/` — Documentation Site

```text
docs/
├── assets/
│   └── images/
│       ├── favicon.ico
│       └── logo.png
├── config/
│   ├── _config.yml        # Jekyll site configuration
│   ├── .markdownlint.yml  # Markdown lint rules
│   ├── Gemfile
│   └── Gemfile.lock
└── pages/
    ├── index.md
    └── software/
        ├── index.md
        ├── getting_started.md
        ├── project_structure.md  ← you are here
        └── style_guide.md
```

The site is built with [Jekyll](https://jekyllrb.com/) using the
[Just the Docs](https://just-the-docs.com/) theme. Source pages live under
`docs/pages/`; static assets (logo, favicon) go in `docs/assets/images/`.
Markdown style rules are enforced by `.markdownlint.yml` and checked in the
pre-commit hook.

To preview the site locally:

```bash
./cresco.sh serve_docs
```
