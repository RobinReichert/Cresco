---
title: Hardware tests
layout: default
parent: Software
nav_order: 5
---

# Hardware tests

Some behaviour can only be checked on real hardware, for example that the
captive portal is reachable over Wi-Fi and accepts credentials. These tests run
in CI on a self-hosted runner that has the board attached.

## Overview

The `test-hw` job in `.github/workflows/check.yml` runs when `cross/`,
`logic/`, `.cargo/`, `.github/` or `rust-toolchain.toml` changed. It needs the
`check-rust` job, because it flashes the firmware that job built.

The rig consists of two ESP32-C3 boards plugged into the runner

![Image]({{site.baseurl}}/assets/drawio/hw_test_setup.svg)

The test client is a second board because the device under test hosts the
access point. Something else has to join it as a Wi-Fi client and talk to the
captive portal. The tests are plain MicroPython scripts that run on the client
board.

## What the job does

1. **Download firmware:** fetches the firmware artifact from `check-rust`.
2. **Reset:** `probe-rs erase --chip esp32c3` wipes the device under test. The
   captive portal only starts when no Wi-Fi credentials are stored, so every
   run starts from a clean device.
3. **Flash:** `tests/flash.py` runs `./cresco.sh flash app_embedded`, which
   downloads the firmware with `probe-rs` and resets the board.
4. **Run the test:** `tests/mp_run.py` runs a script on the client board.

## Running a test script

```bash
uv run python tests/mp_run.py tests/client/join.py
```

`tests/mp_run.py` runs `mpremote connect "$MP_PORT" run <script>` and prints the
output as it arrives. `mpremote` does not return the exit code of the script,
so the runner decides by the output. A test passes if the script prints a line
that is exactly `PASS`. Anything else counts as a failure: a `FAIL` line, a
traceback, a crash or no output.

## Tests

| Script | What it checks |
| --- | --- |
| `tests/client/dns.py` | The portal's DNS answers every name with its own IP |
| `tests/client/join.py` | The portal accepts credentials |

The order matters. `dns.py` needs the portal, which only exists until `join.py`
has provisioned the device, so it runs first. `join.py` still runs if the DNS
test fails, because the two do not depend on each other.

## Setup

### 1. Flash MicroPython on the client board

Download the `ESP32_GENERIC_C3` image from
[micropython.org](https://micropython.org/download/), then erase and flash it.
The offset is `0x0` for the C3.

```bash
espflash erase-flash --port <port>
espflash write-bin --port <port> 0x0 <micropython image>.bin
```

The firmware stays on the board across resets. Flash it again only to update
MicroPython or after an erase.

### 2. Add the Wi-Fi credentials

The credentials live on the client board and not in the repository. Create
`wifi_credentials.py`:

```python
SSID = "your-network-name"
PASSWORD = "your-password"
```

Copy it to the board:

```bash
uv run mpremote connect <port> cp wifi_credentials.py :wifi_credentials.py
```
