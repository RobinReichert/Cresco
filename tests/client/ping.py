import network
import socket
import time

TIMEOUT = 30


def main() -> int:
    wlan = network.WLAN(network.STA_IF)
    wlan.active(True)
    wlan.connect("Cresco")
    start = time.time()
    while not wlan.isconnected():
        if start < time.time() - TIMEOUT:
            print("timeout while connecting to wifi")
            wlan.active(False)
            return 1
        time.sleep(1)
    print("successfully connected to wifi")
    s = socket.socket()
    s.settimeout(10)
    try:
        s.connect(("192.168.1.1", 80))
        print("successfully connected to socket")
        return 0
    except OSError:
        return 1
    finally:
        s.close()
        wlan.active(False)


if __name__ == "__main__":
    err = main()
    print("FAIL" if err else "PASS")
    if err:
        raise SystemExit(1)
