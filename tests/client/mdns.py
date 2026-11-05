import network
import socket
import time

import wifi_credentials

TIMEOUT = 30
ATTEMPTS = 30
QUERY = (
    b"\x12\x34"  # id
    b"\x00\x00"  # flags: standard query
    b"\x00\x01"  # 1 question
    b"\x00\x00\x00\x00\x00\x00"  # no answer, authority or extra records
    b"\x06cresco\x05local\x00"  # name as length-prefixed labels, 0 ends it
    b"\x00\x01"  # type A
    b"\x00\x01"  # class IN
)


def main() -> int:
    wlan = network.WLAN(network.STA_IF)
    wlan.active(True)
    wlan.connect(wifi_credentials.SSID, wifi_credentials.PASSWORD)
    start = time.time()
    while not wlan.isconnected():
        if start < time.time() - TIMEOUT:
            print("timeout while connecting to wifi")
            wlan.active(False)
            return 1
        time.sleep(1)
    print("successfully connected to wifi")
    s = None
    try:
        ip, mask, _, _ = wlan.ifconfig()
        # Sent from a random port, so this is a legacy unicast query (RFC 6762
        # section 6.7) and the answer comes back to this socket directly.
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.settimeout(1)
        # The device may still be joining the network, so ask again like a
        # real resolver would.
        data = None
        resolved = None
        # Keep going until both the raw query and the resolver have an answer,
        # or the attempts are used up.
        for attempt in range(ATTEMPTS):
            if data is None:
                s.sendto(QUERY, ("224.0.0.251", 5353))
                try:
                    data, _ = s.recvfrom(512)
                    print("raw query answered, attempt", attempt + 1)
                except OSError:
                    print("no answer to the raw query, attempt", attempt + 1)
            if resolved is None:
                try:
                    resolved = socket.getaddrinfo("cresco.local", 80)[0][-1][0]
                    print("getaddrinfo ->", resolved)
                except OSError as e:
                    print("getaddrinfo failed:", e)
            if data is not None and resolved is not None:
                break
            time.sleep(1)
        if data is None:
            print("no answer to the raw query after", ATTEMPTS, "attempts")
            return 1
        print(data)
        # Informational only, the result depends on the raw query.
        if resolved is None:
            print("getaddrinfo never resolved cresco.local")
        if data[:2] != QUERY[:2]:
            print("response id does not match the query")
            return 1
        if not data[2] & 0x80:
            print("not a response")
            return 1
        if data[6:8] != b"\x00\x01":
            print("expected exactly one answer")
            return 1
        addr = ".".join(str(b) for b in data[-4:])
        print("cresco.local ->", addr)
        in_subnet = all(
            int(a) & int(m) == int(b) & int(m)
            for a, b, m in zip(ip.split("."), addr.split("."), mask.split("."))
        )
        if not in_subnet:
            print("answer is not in the local subnet")
            return 1
        return 0
    except OSError as e:
        print(e)
        return 1
    finally:
        if s:
            s.close()
        wlan.disconnect()
        wlan.active(False)


if __name__ == "__main__":
    err = main()
    print("FAIL" if err else "PASS")
