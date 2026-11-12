import argparse
from ipaddress import IPv4Address
import sys
import shutil
import time
from typing import Callable

from selenium import webdriver
from selenium.webdriver.remote.webdriver import WebDriver

from tests.mp_run import Run, run
from tests.web.dashboard import DashboardPage

PAGE_LOAD_TIMEOUT = 30
VOLTAGE_TIMEOUT = 10

type TestFunc = Callable[[WebDriver, IPv4Address], None]


def set_voltage(runner: Run, name: str, volts: float) -> None:
    runner.send(f"set {name} {volts}")
    deadline = time.monotonic() + VOLTAGE_TIMEOUT
    while f"OK {name} {volts}" not in runner.output:
        assert time.monotonic() < deadline, f"no confirmation for {name} {volts}"
        time.sleep(0.05)


def is_close(actual: float | str, expected: float, tolerance: float) -> bool:
    value = float(actual)
    return abs(value - expected) <= tolerance


def dashboard_integrity_test(driver: WebDriver, address: IPv4Address) -> None:
    dashboard = DashboardPage.open(driver, str(address))
    title = dashboard.get_title()
    assert title == "Cresco Dashboard", f"unexpected title: {title!r}"
    ec = dashboard.get_ec()
    assert ec == "Not calibrated", f"unexpected ec: {ec!r}"
    ph = dashboard.get_ph()
    assert ph == "Not calibrated", f"unexpected ph: {ph!r}"
    settings = dashboard.open_settings()
    dashboard = settings.close_settings()
    settings = dashboard.open_settings()
    calibrate_ec = settings.goto_calibrate_ec()
    title = calibrate_ec.get_title()
    assert title == "Cresco Calibration", f"unexpected title: {title!r}"
    runner = run("tests/client/set_voltage.py")
    calibrate_ec.start()
    set_voltage(runner, "ec", 1.0)
    calibrate_ec.submit_point(4.0)
    set_voltage(runner, "ec", 3.0)
    calibrate_ec.submit_point(6.0)
    dashboard = calibrate_ec.finish()
    title = dashboard.get_title()
    assert title == "Cresco Dashboard", f"unexpected title: {title!r}"
    settings = dashboard.open_settings()
    calibrate_ph = settings.goto_calibrate_ph()
    title = calibrate_ph.get_title()
    assert title == "Cresco Calibration", f"unexpected title: {title!r}"
    calibrate_ph.start()
    set_voltage(runner, "ph", 0.5)
    calibrate_ph.submit_point(1.0)
    set_voltage(runner, "ph", 2.5)
    calibrate_ph.submit_point(2.0)
    dashboard = calibrate_ph.finish()
    title = dashboard.get_title()
    assert title == "Cresco Dashboard", f"unexpected title: {title!r}"
    for _ in range(
        3
    ):  # we need to send the message repeatedly so it does not timeout. the first measurement can take up to 60 seconds.
        time.sleep(20)
        set_voltage(runner, "ec", 2.0)
        set_voltage(runner, "ph", 1.5)
    dashboard.refresh()
    ec = dashboard.get_ec()
    ph = dashboard.get_ph()
    assert is_close(ec, 5.0, 0.2), f"unexpected ec of: {ec!r}, should have been 5.0"
    assert is_close(ph, 1.5, 0.2), f"unexpected ph of: {ph!r}, should have been 1.5"
    print(f"ec: {ec} is close to actual 5.0")
    print(f"ph: {ph} is close to actual 1.5")
    runner.send("exit")
    runner.wait()
    assert runner.passed


TEST_FUNCTIONS: dict[str, TestFunc] = {"dashboard_integrity": dashboard_integrity_test}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("ip", help="IP address of the device")
    parser.add_argument("test_name", help="Name of the test to be run")
    parser.add_argument("--headed", action="store_true", help="show the browser window")
    args = parser.parse_args()
    options = webdriver.ChromeOptions()
    chromium = shutil.which("chromium")
    if chromium:
        options.binary_location = chromium
    if not args.headed:
        options.add_argument("--headless=new")
    options.page_load_strategy = "eager"
    driver = webdriver.Chrome(options=options)
    driver.set_page_load_timeout(PAGE_LOAD_TIMEOUT)
    try:
        TEST_FUNCTIONS[args.test_name](driver, args.ip)
        return 0
    except Exception as e:
        print(e)
    finally:
        driver.quit()
    return 1


if __name__ == "__main__":
    err = main()
    print("FAIL" if err else "PASS")
    sys.exit(err)
