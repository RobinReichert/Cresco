---
title: Dhcp
layout: default
parent: Modules
nav_order: 1
---

# Dhcp

## Dhcp Server

The `DhcpServer` trait and `DhcpAction` is the interface between
the cross compiled part and the logic.

```rust
pub enum DhcpAction {
    SendPacket {
        payload: [u8; 576],
        len: usize,
        remote: Ipv4Addr
    },
    Ignore,
}

pub trait DhcpServer {
    fn handle_message(
        &mut self,
        buffer: &[u8],
        time: u64,
    ) -> DhcpAction;
}
```

## Simple Dhcp Server

`SimpleDhcpServer` is a minimal, single-client DHCP
server implemented as a finite state machine. It
supports the full DORA handshake
(Discover → Offer → Request → Ack), lease renewal,
release, and lease expiry.
The address is dealt in a **first come first serve** manner.

![Image]({{site.baseurl}}/assets/drawio/dhcp_fsm.svg)

{: .warning}
> **Single client only.** The server tracks one lease at a time.
> A second device cannot obtain an address until
> the existing lease expires or is released.
