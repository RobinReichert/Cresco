import network
import socket
import time

TIMEOUT = 30
PORTAL_IP = "192.168.1.1"


def main() -> int:
    wlan = network.WLAN(network.STA_IF)
    wlan.active(True)
    try:
        wlan.connect("Cresco")
        start = time.time()
        while not wlan.isconnected():
            if start < time.time() - TIMEOUT:
                print("timeout while connecting to wifi")
                return 1
            time.sleep(1)
        print("successfully connected to wifi")
        addr = socket.getaddrinfo("www.example.com", 80)[0][-1][0]
        print("www.example.com ->", addr)
        if addr != PORTAL_IP:
            print("expected the portal ip")
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
