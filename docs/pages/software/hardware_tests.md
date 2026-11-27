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
4. **Run the test:** `tests/tests.py` runs a test by name. A client test
   runs its script on the client board with `run()` from `tests/mp_run.py`.
5. **Check the dashboard:** a browser test opens the dashboard of the device
   and calibrates the sensors with voltages from the client board, see
   [Web tests].

## Running a test

```bash
uv run python -m tests.tests join
```

`run()` in `tests/mp_run.py` soft-resets the board on `$MP_PORT`, runs the
script with the `mpremote` library and prints the output as it arrives.
The library does not
return the exit code of the script, so the runner decides by the output.
A test passes if the script prints a line that is exactly `PASS`.
Anything else counts as a failure: a `FAIL` line, a traceback,
a crash or no output.

## Passing values to later steps

A script can hand a value to later workflow steps by printing a line
`OUTPUT <name>=<value>`. `tests/mp_run.py` appends it to the file in
`GITHUB_OUTPUT`, so it becomes a step output. For example `mdns_resolver.py`
prints `OUTPUT ip=<address>`, and a later step reads it as
{% raw %}`${{ steps.mdns_resolver.outputs.ip }}`{% endraw %} (the step needs an
`id`).

## Tests

| Script | What it checks |
| --- | --- |
| `dns.py` | The portal's DNS answers every name with its own IP |
| `join.py` | The portal accepts credentials |
| `mdns_legacy.py` | A query from a random port is answered to the sender |
| `mdns_resolver.py` | `getaddrinfo("cresco.local")` resolves to an address |
| `mdns_normal.py` | A query from port 5353 is answered to the multicast group |

The order matters. `dns.py` needs the portal, which only exists until `join.py`
has provisioned the device, so it runs first. `join.py` still runs if the DNS
test fails, because the two do not depend on each other. The mDNS tests need
the device on the Wi-Fi network, so they only run after a successful join. They
do not depend on each other. The [Web tests] run last. They need the IP of the
device, which `mdns_resolver.py` hands over.

`mdns_normal.py` binds port 5353 before it connects, because the port is taken
as soon as the client board is on the network.

## Adding a test

**Client test:**

1. Write the script in `tests/client/` and make it print `PASS` or `FAIL` at
   the end, like the existing ones.
2. In `tests/tests.py`, add a function that runs it and register it:

   ```python
   def my_test(args: argparse.Namespace) -> int:
       return run("tests/client/my_test.py").wait()

   TEST_FUNCTIONS = {
       ...
       "my_test": Test(my_test),
   }
   ```

3. Add a step to the `test-hw` job that runs
   `uv run python -m tests.tests my_test`.

**Web test:** write a function `(driver, address)` that raises on failure and
register it with `Test(run_web(my_web_test), web_args)`. `run_web` creates the
browser and `web_args` adds the `ip` and `--headed` arguments, see
[Web tests].

**Extra arguments:** the second field of `Test` is a function that adds
arguments to the subparser of that test. Write one like `web_args` and read the
values from `args` in the test function. Tests without extra arguments need
none.

## Debug API

`tests/debug/` reads and writes static variables of the device under test with
`probe-rs gdb` and gdb.

```python
from tests.debug import symbols
from tests.debug.debugger import Debugger

with Debugger() as debugger:
    targets = debugger.read(symbols.TARGETS.value)
```

- `read` returns the value as nested dicts. Leaves are strings as gdb prints
  them, for example `{'ph': {'Constant': {'c': '5'}}, ...}`. An enum shows up
  as `{variant: fields}`.
- `write(expr, value)` assigns `value` in gdb's syntax to the variable at
  `expr`. No test uses it yet.
- The chip is halted for the whole `with` block, and gdb detaches when it
  ends. probe-rs 0.32 cannot halt it a second time in one session, so open a
  new block for a fresh value.

`tests/debug/symbols.py` holds the paths to the variables. A wrapper type such
as `Mutex` gets a class that knows its layout, and `value` leads to the
content. Add a variable there before a test uses it:

```python
MEASUREMENTS = Mutex("cross::blackboard::MEASUREMENTS")
```

Check that the firmware does not hold a lock before writing with
`debugger.read(symbols.TARGETS.locked) == "false"`. Writing to a `Signal` or
`Channel` does not wake the task that waits on it.

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

[Web tests]: {{ site.baseurl }}{% link pages/software/web_tests.md %}
