---
title: Wifi 
layout: default
parent: Modules
nav_order: 3
---

# Wifi

The WiFi module provides a state-machine-driven approach to managing
WiFi connectivity on embedded devices.
It separates the *manager* of how WiFi should behave (the state machine)
from the *mechanism* that carries it out (the async connection loop).

---

## Overview

The module is structured around three concepts:

- **Events** — things that happen in the world (`WifiEvent`)
- **Actions** — things the driver should do in response (`WifiAction`)
- **Manager** — the state machine that maps events to
actions (`WifiManager` trait)

This separation means the state machine logic can be tested
entirely without hardware or async runtimes.

---

## State Machine

### States

The built-in `SimpleWifiManager` moves through four states:

| State | Meaning |
| --- | --- |
| `RetrievingCredentials` | Checking storage for saved WiFi credentials |
| `WifiApStaStarted` | Running a portal to collect credentials from the user |
| `WifiClientStarted` | Attempting to connect to an access point |
| `Connected` | Successfully connected; waiting for a disconnect event |

### State Diagram

![Image]({{site.baseurl}}/assets/drawio/wifi_manager_fsm.svg)

### `WifiEvent`

Events are produced by the driver and fed into the manager via `handle_event`.

| Variant | When to fire |
| --- | --- |
| `CredentialsMissing` | No stored credentials were found at startup |
| `CredentialsFound` | Stored credentials were successfully loaded |
| `CredentialsReceived` | User submitted credentials via the captive portal |
| `Connected` | The station successfully associated with an AP |
| `NotConnected` | A connection attempt timed out or was refused |
| `Disconnected` | The station lost an existing connection |
| `Timeout` | The captive portal window expired without receiving credentials |

### `WifiAction`

Actions are returned by `handle_event` and tell the driver what to do next.

| Variant | Meaning |
| --- | --- |
| `Ignore` | The event was not valid in the current state; do nothing |
| `RetrieveCredentials` | Load credentials from persistent storage |
| `StartWifiApSta` | Start AP+STA mode and open the captive portal |
| `StartWifiClient` | Connect to the given SSID with the given password |
| `WaitForDisconnect` | Connection is up; block until a disconnect event |

### `WifiManager` trait

```rust
pub trait WifiManager{
    fn handle_event(&mut self, event: WifiEvent) -> WifiAction;
}
```
