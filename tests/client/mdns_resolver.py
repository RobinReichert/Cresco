import network
import socket
import time

import wifi_credentials

TIMEOUT = 30
ATTEMPTS = 30


def main() -> int:
    wlan = network.WLAN(network.STA_IF)
    wlan.active(True)
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
        addr = None
        # The device may still be joining the network, so try again.
        for attempt in range(ATTEMPTS):
            try:
                addr = socket.getaddrinfo("cresco.local", 80)[0][-1][0]
                break
            except OSError as e:
                print("lookup failed, attempt", attempt + 1, e)
                time.sleep(1)
        if addr is None:
            print("could not resolve cresco.local after", ATTEMPTS, "attempts")
            return 1
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
        wlan.active(False)


if __name__ == "__main__":
    err = main()
    print("FAIL" if err else "PASS")
