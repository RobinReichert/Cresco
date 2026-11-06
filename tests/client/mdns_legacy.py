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
    s = None
    try:
        wlan.connect(wifi_credentials.SSID, wifi_credentials.PASSWORD)
        start = time.time()
        while not wlan.isconnected():
            if start < time.time() - TIMEOUT:
                print("timeout while connecting to wifi")
                return 1
            time.sleep(1)
        print("successfully connected to wifi")
        ip, mask, _, _ = wlan.ifconfig()
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.settimeout(1)
        data = None
        for attempt in range(ATTEMPTS):
            s.sendto(QUERY, ("224.0.0.251", 5353))
            try:
                data, _ = s.recvfrom(512)
                break
            except OSError:
                print("no answer, attempt", attempt + 1)
        if data is None:
            print("no answer after", ATTEMPTS, "attempts")
            return 1
        print("data: ", data)
        addr = ".".join(str(b) for b in data[-4:])
        print("cresco.local ->", addr)
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
