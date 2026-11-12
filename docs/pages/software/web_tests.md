---
title: Web tests
layout: default
parent: Software
nav_order: 6
---

# Web tests

The dashboard is served by the device, and some of its behaviour only shows in
a browser: that the values load, that the settings drawer opens and that the
calibration walks through its steps. The web tests drive the real dashboard in
a browser with [Selenium](https://www.selenium.dev/). They run in CI as the last
step of the [hardware tests] and can be run locally against any device.

## Running a test

Run it from the repository root, as a module:

```bash
uv run python -m tests.web.tests <ip> <test name> [--headed]
```

| Argument | Meaning |
| --- | --- |
| `ip` | Address of the device |
| `test name` | A key of `TEST_FUNCTIONS` in `tests/web/tests.py` |
| `--headed` | Show the browser window, by default it runs headless |

For example:

```bash
uv run python -m tests.web.tests 192.168.1.50 dashboard_integrity --headed
```

The runner prints `PASS` or `FAIL` and exits with 0 or 1. If a test fails, the
message of the error is printed before `FAIL`.

Do not run the file as a script (`python tests/web/tests.py`). The tests import
each other as `tests.web...`, which only works when the repository root is on
the import path, and `-m` takes care of that.

## Tests

| Test | What it checks |
| --- | --- |
| `dashboard_integrity` | The pages load and the calibration flow works |

`dashboard_integrity` goes through the dashboard like a user:

1. Opens the dashboard, checks its title and checks that pH and EC read
   `Not calibrated`. This only holds on a device without calibration, which CI
   guarantees by erasing the device first.
2. Opens and closes the settings drawer.
3. Opens the EC calibration from the drawer, checks the page title and runs
   the calibration. The client board applies 1.0 V for the reference value 4.0
   and 3.0 V for 6.0.
4. Returns to the dashboard and does the same for pH: 0.5 V for 1.0 and 2.5 V
   for 2.0.
5. Applies 2.0 V to EC and 1.5 V to pH for one minute, reloads the dashboard
   and checks that the values are close to the ones the calibration predicts:
   EC 5.0 and pH 1.5, with a tolerance of 0.2. The first measurement after a
   calibration can take up to 60 seconds, hence the wait.

The test needs the client board, see [Voltage source]. It reads the serial port
from `MP_PORT`, which the CI job sets. Locally, export it first:

```bash
export MP_PORT=<port>
```

## Voltage source

A calibration needs two different readings per sensor. The test gets them from
the client board of the [hardware tests], which applies a voltage to the ADC
pins of the device. The ESP32-C3 has no DAC, so the client uses PWM and an RC
low-pass filter to turn it into a steady voltage.

![Image]({{site.baseurl}}/assets/drawio/voltage_source.svg)

| Client | Cresco | Signal |
| --- | --- | --- |
| `GPIO4` | `GPIO0` | pH, through 10 kΩ and 1 µF |
| `GPIO3` | `GPIO1` | EC, through 10 kΩ and 1 µF |
| `GND` | `GND` | Common ground |

The resistor sits in series with the signal, the capacitor between the Cresco
pin and ground. The grounds of both boards must be connected, otherwise the
voltage has no reference.

The filter has a time constant of 10 ms (10 kΩ × 1 µF). The PWM runs at
5 kHz, far above the cut-off frequency of about 16 Hz, so little ripple is left
on the output. The voltage settles within about 50 ms, and the script waits
0.5 s after every change to be safe.

[hardware tests]: {{ site.baseurl }}{% link pages/software/hardware_tests.md %}
[Voltage source]: #voltage-source
