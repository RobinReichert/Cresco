---
title: Web
layout: default
parent: Modules
nav_order: 4
---

# Web

The web module handles the http web app.
It contains a **captive_app** that is used to get the wifi credentials
of the network to be joined.

**Structure**
The tasks are spawned in the main function.

{: .note}
> State synchronization is handled via embassy sync.

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

### Related

- [Wifi Module]({{ site.baseurl }}{% link pages/software/modules/wifi.md %})
