import network
import socket
import time

import wifi_credentials

TIMEOUT = 30
QUERY = (
    b"\x00\x00"  # id
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
        ip = wlan.ifconfig()[0]
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind(("0.0.0.0", 5353))
        group = bytes([224, 0, 0, 251]) + bytes(int(x) for x in ip.split("."))
        s.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, group)
        s.settimeout(3)
        s.sendto(QUERY, ("224.0.0.251", 5353))
        data, _ = s.recvfrom(512)
        print(data)
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
