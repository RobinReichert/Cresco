---
title: Web
layout: default
parent: Modules
nav_order: 4
---

# Web

The web module serves two separate
[picoserve](https://github.com/sammhicks/picoserve) http apps:

| App | Network | Purpose |
| --- | --- | --- |
| **captive_app** | access point | receives the wifi credentials |
| **cresco_app** | joined network | dashboard and calibration |

**Structure**
Each app has its own task pool (`CAPTIVE_APP_TASK_POOL_SIZE`,
`CRESCO_APP_TASK_POOL_SIZE`). The tasks are spawned in the main function.
Both apps listen on port 80, each on its own network stack.

{: .note}
> State synchronization is handled via embassy sync.

---

## Captive Portal

The app is only available during startup if no credentials are stored,
or if the connection to present credentials fails.
It is used to receive wifi credentials from the user.

### Routes

| route | type | behaviour |
| --- | --- | --- |
| `/` | `GET` | returns the captive page |
| `/ssids` | `GET` | returns a list of available ssids |
| `/login` | `POST` | posts the entered login credentials |
| `/*` | `*` | redirect any route to `/` |

### Shared State

![Image]({{site.baseurl}}/assets/drawio/captive_app_shared_state.svg)

---

## Cresco App

The app is reachable on the joined network and serves the user interface for
the running system.

### Pages and Assets

| route | type | behaviour |
| --- | --- | --- |
| `/` | `GET` | dashboard: live pH and EC, settings drawer |
| `/calibrate/ph` | `GET` | calibration wizard for pH |
| `/calibrate/ec` | `GET` | calibration wizard for EC |
| `/style.css` | `GET` | compiled stylesheet |
| `/plant.webp` | `GET` | dashboard image |

`/calibrate/ph` and `/calibrate/ec` serve the same page. The page reads the
sensor from its own path.

### API

| route | type | behaviour |
| --- | --- | --- |
| `/api/measurements` | `GET` | latest pH and EC as json |
| `/api/calibration/status` | `GET` | current calibration status |
| `/api/calibration/start` | `POST` | start a calibration |
| `/api/calibration/point` | `POST` | take a calibration point |
| `/api/calibration/abort` | `POST` | abort the running calibration |

The `POST` routes take a url-encoded form:

- `start`: `sensor=ph` or `sensor=ec`
- `point`: `sensor=ph` or `sensor=ec`, and `actual=<value>`, the known
  value of the solution the probe is in
- `abort`: no body

**Measurements.** The response is `{ "ph": .., "ec": .. }`. A value is `null`
while its sensor is not calibrated or has not been measured since. The values
come from the blackboard, which the measurement task only writes once **both**
sensors are calibrated and a periodic measurement (every 60 s) has run.

**Commands.** `start`, `point` and `abort` do not wait for the result. They
put a `MeasurementCommand` into a channel of capacity 1 and answer
immediately.

| status | meaning |
| --- | --- |
| `200` | the command was queued |
| `400` | the form could not be decoded |
| `503` | the channel is full, the device is busy; try again |

A `200` only means "accepted". A `point` or `abort` while no calibration is
waiting for it is accepted but has no effect.

**Status.** The result of a command is read from `/api/calibration/status`,
which returns `{ "status": .. }` with a `MeasurementStatus` in `snake_case`:

| status | meaning |
| --- | --- |
| `not_calibrated_yet` | initial value, or a sensor was found uncalibrated |
| `awaiting_first_point` | calibration started, waiting for point 1 |
| `awaiting_second_point` | point 1 taken, waiting for point 2 |
| `probe_read_failed` | the probe could not be read; calibration ended |
| `calibration_failed` | the points could not be fitted; calibration ended |
| `calibration_succeeded` | both points taken, the sensor is calibrated |
| `calibration_aborted` | aborted, or an unexpected command arrived |

### Calibration Page

The page is a four step wizard: **Start**, **Point 1 of 2**, **Point 2 of 2**,
**Done**. Each step sends one command and then polls the status (every 300 ms,
giving up after 15 s) until the status leaves the state it was waiting in:

| step | command | waits until |
| --- | --- | --- |
| Start | `start` | `awaiting_first_point` |
| Point 1 | `point` | `awaiting_second_point` |
| Point 2 | `point` | `calibration_succeeded` |

If the status becomes `probe_read_failed`, `calibration_failed` or
`calibration_aborted` instead, the page shows an error and returns to the
start step. When the user leaves the page mid-calibration (cancel, back,
reload, closing the tab) it sends `abort` with `navigator.sendBeacon`, so the
device is not left waiting for a point.

{: .note}
> Calibration is held in RAM only. After a reboot both sensors are
> uncalibrated again and the dashboard shows *Not calibrated*.

### Dashboard

The dashboard fetches `/api/measurements` on load and then every 10 s. A
`null` value is shown as *Not calibrated*. If a request fails, the last
values stay on screen. The *Dosing pump* and *Last reading* rows are static
placeholders and not yet backed by data.

### Related

- [Wifi Module]({{ site.baseurl }}{% link pages/software/modules/wifi.md %})
- [Blackboard]({{ site.baseurl }}{% link pages/software/modules/blackboard.md %})
