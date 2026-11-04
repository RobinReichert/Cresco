import network
import socket
import time
import wifi_credentials


def try_bind(label):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind(("0.0.0.0", 5353))
        print(label, "bind OK")
    except OSError as e:
        print(label, "bind failed:", e)
    finally:
        s.close()


try_bind("fresh, wifi off")
wlan = network.WLAN(network.STA_IF)
wlan.active(True)
try_bind("wifi active")
wlan.connect(wifi_credentials.SSID, wifi_credentials.PASSWORD)
while not wlan.isconnected():
    time.sleep(1)
try_bind("wifi connected")
print(network.hostname())
wlan.active(False)
