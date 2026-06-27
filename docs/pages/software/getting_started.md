---
title: Getting Started
layout: default
parent: Software
nav_order: 1
---

# Getting Started

## Prerequisites

### 1. Clone the Repository

```bash git clone <repo-url>
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

### 4. Install uv

If you already have Cargo installed, you can install uv with:

```bash
cargo install uv
```

Otherwise follow
the [official uv installation guide](https://docs.astral.sh/uv/getting-started/installation/)
for your platform.

### 5. Install ESP-IDF (camera firmware)

The camera firmware is built with
[ESP-IDF](https://docs.espressif.com/projects/esp-idf/en/v6.0.1/esp32s3/get-started/index.html)
v6.0.1 for the ESP32-S3. The recommended way to install it is the
[ESP-IDF Installation Manager (EIM)](https://github.com/espressif/idf-im-ui),
Espressif's official installer, which sets up the framework, the toolchain
and a dedicated Python environment.

{: .note}
> ESP-IDF is only required for the `build camera` command. If you are only
> working on the embedded Rust application you can skip this step.

After installing, note the command that activates the environment in your shell
(e.g. an `export.sh` or an EIM `activate_idf_*.sh` script) — you will need it in
the setup step below.

---

## Setup

{: .note}
> The project ships two equivalent task runners: `cresco.sh` for Linux and macOS
> and `cresco.ps1` for Windows (PowerShell). Use whichever matches your platform
> the commands and arguments are identical.

### 1. Install tooling and git hooks

Once all prerequisites are in place, run:

```bash
# Linux / macOS
./cresco.sh setup
```

```powershell
# Windows (PowerShell)
.\cresco.ps1 setup
```

### 2. Configure ESP-IDF activation (camera firmware only)

The `build camera` command needs to activate ESP-IDF.
Because every developer's install path differs, the activation command lives in
an untracked local config instead of being committed. Create it from the
template:

```bash
# Linux / macOS
cp cresco.local.sh.example cresco.local.sh
```

```powershell
# Windows (PowerShell)
Copy-Item cresco.local.ps1.example cresco.local.ps1
```

Then set `IDF_EXPORT_CMD` to the command that activates ESP-IDF on your machine:

```bash
# cresco.local.sh
IDF_EXPORT_CMD="source $HOME/.espressif/tools/activate_idf_v6.0.1.sh"
```

```powershell
# cresco.local.ps1
$IDF_EXPORT_CMD = "& $HOME\.espressif\tools\activate_idf_v6.0.1.ps1"
```
