import select
import sys
import time

from machine import PWM, Pin

TIMEOUT = 30
PINS = {
    "ph": 4,
    "ec": 5,
}
VCC = 3.3
PWM_FREQ = 5000


def set_voltage(pwms: dict, args: list[str]) -> None:
    name, volts = args
    if name not in pwms:
        raise ValueError("unknown pin " + name)
    volts = float(volts)
    if not 0 <= volts <= VCC:
        raise ValueError("voltage out of range")
    pwms[name].duty_u16(int(volts / VCC * 65535))
    time.sleep(0.5)
    print("OK", name, volts)


COMMANDS = {
    "set": set_voltage,
}


def main() -> int:
    pwms = {
        name: PWM(Pin(gpio), freq=PWM_FREQ, duty_u16=0) for name, gpio in PINS.items()
    }
    try:
        print("READY")
        poll = select.poll()
        poll.register(sys.stdin, select.POLLIN)
        while True:
            if not poll.poll(TIMEOUT * 1000):
                print("timeout while waiting for a command")
                return 1
            parts = sys.stdin.readline().split()
            if not parts:
                continue
            if parts[0] == "exit":
                return 0
            try:
                COMMANDS[parts[0]](pwms, parts[1:])
            except (KeyError, ValueError) as e:
                print("ERR", parts[0], e)
                return 1
    finally:
        for pwm in pwms.values():
            pwm.deinit()


if __name__ == "__main__":
    err = main()
    print("FAIL" if err else "PASS")
