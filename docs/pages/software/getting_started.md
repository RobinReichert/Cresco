---
title: Getting Started 
layout: default
parent: Software
nav_order: 1
---

# Getting Started

## Prerequisites

### 1. Clone the Repository

```bash
git clone <repo-url>
cd <repo-name>
```

### 2. Install Docker

If not already installed go ahead and install Docker from the
[official website](https://docs.docker.com/get-docker/) for your platform.

{: .note}
> Docker is used to keep the installations as few as possible - it runs
> things like linting checks

### 3. Install Rust Toolchain

Follow the [Espressif Rust toolchain guide][toolchain-guide-link] for RISC-V targets.
You'll need to install `rustup`, add the `riscv32imc-unknown-none-elf` target (ESP32-C3),
and install `probe-rs` for flashing.

[toolchain-guide-link]: https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html

{: .tip}
> Although mainly used to set up the project, installing esp-generate may also
> prove useful if you plan to change the project's configuration later on.

---

## Setup

Once all prerequisites are in place, run:

```bash
./cresco.sh setup
```
