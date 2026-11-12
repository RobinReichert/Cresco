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

1. Opens the dashboard and checks its title.
2. Opens and closes the settings drawer.
3. Opens the EC calibration from the drawer, checks the page title and runs
   the calibration with the reference values 4.0 and 6.0.
4. Returns to the dashboard and does the same for pH with 1.0 and 2.0.

[hardware tests]: {{ site.baseurl }}{% link pages/software/hardware_tests.md %}
