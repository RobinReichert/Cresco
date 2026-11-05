import network
import socket
import time

import wifi_credentials

TIMEOUT = 30
ATTEMPTS = 30
NAME = b"\x06cresco\x05local\x00"
QUERY = (
    b"\x00\x00"  # id: mDNS queries use 0
    b"\x00\x00"  # flags: standard query
    b"\x00\x01"  # 1 question
    b"\x00\x00\x00\x00\x00\x00"  # no answer, authority or extra records
    + NAME
    + b"\x00\x01"  # type A
    b"\x00\x01"  # class IN
)


def wait_for_answer(s):
    # The socket may also receive our own query and traffic of other devices.
    for _ in range(10):
        try:
            data, _ = s.recvfrom(512)
        except OSError:
            return None
        if len(data) > 12 and data[2] & 0x80 and NAME in data:
            return data
    return None


def main() -> int:
    wlan = network.WLAN(network.STA_IF)
    wlan.active(True)
    s = None
    try:
        # Once connected, the port is already taken on this board, so bind it
        # first. This makes it a normal mDNS query: sent from port 5353 and
        # answered to the multicast group.
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind(("0.0.0.0", 5353))
        wlan.connect(wifi_credentials.SSID, wifi_credentials.PASSWORD)
        start = time.time()
        while not wlan.isconnected():
            if start < time.time() - TIMEOUT:
                print("timeout while connecting to wifi")
                return 1
            time.sleep(1)
        print("successfully connected to wifi")
        ip, mask, _, _ = wlan.ifconfig()
        group = bytes([224, 0, 0, 251]) + bytes(int(x) for x in ip.split("."))
        s.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, group)
        s.settimeout(1)
        data = None
        # The device may still be joining the network, so ask again like a
        # real resolver would.
        for attempt in range(ATTEMPTS):
            s.sendto(QUERY, ("224.0.0.251", 5353))
            data = wait_for_answer(s)
            if data is not None:
                break
            print("no answer, attempt", attempt + 1)
        if data is None:
            print("no answer after", ATTEMPTS, "attempts")
            return 1
        print(data)
        if data[:2] != b"\x00\x00":
            print("expected the id to be 0")
            return 1
        if data[4:6] != b"\x00\x00":
            print("a response must not repeat the question")
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
        wlan.active(False)


if __name__ == "__main__":
    err = main()
    print("FAIL" if err else "PASS")
