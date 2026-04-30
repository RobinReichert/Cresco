---
title: Dns
layout: default
parent: Modules
nav_order: 2
---

# Dns

## Dns Server

The `DnsServer` trait and `DnsAction` is the interface between
the cross compiled part and the logic.

```rust
pub enum DnsAction {
    SendPacket {
        payload: [u8; 576],
        len: usize,
    },
    Ignore,
}

pub trait DnsServer {
    fn handle_message(
        &mut self,
        buffer: &[u8],
    ) -> DhcpAction;
}
```

## Message Format

DNS messages are binary-encoded over UDP.
The wire format consists of a fixed 12-byte header,
followed by a variable number of question and answer records.
All multi-byte integers are big-endian.

{: .highlight}
> Everything that is **<span style="color: lightgray;">gray</span>**
in the following tables is not implemented
and only encoded with a default value.

### Header

![Image]({{site.baseurl}}/assets/drawio/dns_header_format.svg)

### Header Flags

![Image]({{site.baseurl}}/assets/drawio/dns_flags_format.svg)

### Question

Each question record contains an encoded name, a 16-bit type field,
and a 16-bit class field
(always `1` for `IN` / Internet in this implementation).

![Image]({{site.baseurl}}/assets/drawio/dns_question_format.svg)

Supported question types:

| Value | Type | Description |
| --- | --- | --- |
| 1 | `A` | IPv4 address |
| 5 | `CNAME` | Canonical name alias |
| 15 | `MX` | Mail exchange |
| 16 | `TXT` | Arbitrary text |
| 28 | `AAAA` | IPv6 address |

### Answer

Answer records extend the question format with a 32-bit TTL
(time-to-live in seconds)
and a 16-bit len of the data, followed by the record data.

![Image]({{site.baseurl}}/assets/drawio/dns_answer_format.svg)

### Name

Domain names are encoded as a sequence of length-prefixed labels,
terminated by a zero byte. For example, `captive.apple.com` encodes as
`[7]captive[5]apple[3]com[0]`. DNS compression pointers
(top two bits `0xC0`) are also supported for parsing
 — they redirect the reader to an earlier offset in the packet.

![Image]({{site.baseurl}}/assets/drawio/dns_name_format.svg)

### Record

Record payloads are type-specific:

![Image]({{site.baseurl}}/assets/drawio/dns_a_record_format.svg)

This is just a example for the IPv4 type.
For other types view the code inside `logic/src/dns.rs`.

## Poisoned Dns Server

The `PoisonedDnsServer` is a simple DNS server designed to support a
**captive portal**. Rather than forwarding or resolving queries,
it responds to every DNS query — regardless of the requested domain - with
an `A` record pointing to `SERVER_IP`.
This forces all DNS-dependent traffic on the network
to resolve to the device running the portal.

**Behaviour:**

- Parses the incoming DNS query.
- Takes the name from the first question record.
- Responds with a single `A` record mapping that name to `SERVER_IP` with
a TTL of `0`.
- Echoes back the original identification,
opcode, and `RD` flag, and sets `AA` and `RA` to `true`.
- Returns `DnsAction::Ignore` if the packet cannot be parsed or contains no questions.

{: .note}
> The TTL of `0` prevents clients from caching the poisoned response,
> which ensures they re-query on each connection attempt - useful
> if the portal IP changes or the device restarts.
