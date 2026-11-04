import network
import socket
import time

import wifi_credentials

TIMEOUT = 30


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
    try:
        print(socket.getaddrinfo("cresco.local", 80))
        return 0
    except OSError as e:
        print(e)
        return 1
    finally:
        wlan.disconnect()
        wlan.active(False)


if __name__ == "__main__":
    err = main()
    print("FAIL" if err else "PASS")
