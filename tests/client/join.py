import network
import socket
import time

import wifi_credentials

TIMEOUT = 30


def main() -> int:
    body = "ssid=%s&password=%s" % (wifi_credentials.SSID, wifi_credentials.PASSWORD)
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
        request = (
            "POST /login HTTP/1.0\r\n"
            "Host: 192.168.1.1\r\n"
            "Content-Type: application/x-www-form-urlencoded\r\n"
            "Content-Length: %d\r\n\r\n%s" % (len(body), body)
        )
        s.send(request.encode())
        for _ in range(10):
            if not wlan.isconnected():
                print("login accepted, device left the portal")
                return 0
            time.sleep(1)
        print("device is still serving the portal")
        return 1
    except OSError as e:
        print(e)
        return 1
    finally:
        s.close()
        wlan.active(False)


if __name__ == "__main__":
    err = main()
    print("FAIL" if err else "PASS")
