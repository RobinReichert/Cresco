import argparse
from ipaddress import IPv4Address
from dataclasses import dataclass
import sys
import shutil
import time
from typing import Callable
import subprocess

from selenium import webdriver
from selenium.webdriver.remote.webdriver import WebDriver

from tests import mp_run
from tests.debug import Debugger
from tests.mp_run import Run, run
from tests.web.dashboard import DashboardPage

PAGE_LOAD_TIMEOUT = 30
VOLTAGE_TIMEOUT = 10


def no_args(parser: argparse.ArgumentParser) -> None:
    pass


def web_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("ip", help="IP address of the device")
    parser.add_argument("--headed", action="store_true", help="show the browser window")


def run_web(
    test: Callable[[WebDriver, IPv4Address], None],
) -> Callable[[argparse.Namespace], int]:
    def run_test(args: argparse.Namespace) -> int:
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
            test(driver, args.ip)
            return 0
        except Exception as e:
            print(e)
            return 1
        finally:
            driver.quit()

    return run_test


@dataclass
class Test:
    run: Callable[[argparse.Namespace], int]
    args: Callable[[argparse.ArgumentParser], None] = no_args


def set_voltage(runner: Run, name: str, volts: float) -> None:
    runner.send(f"set {name} {volts}")
    deadline = time.monotonic() + VOLTAGE_TIMEOUT
    while f"OK {name} {volts}" not in runner.output:
        assert time.monotonic() < deadline, f"no confirmation for {name} {volts}"
        time.sleep(0.05)


def is_close(actual: float | str, expected: float, tolerance: float) -> bool:
    value = float(actual)
    return abs(value - expected) <= tolerance


def storage_test(driver: WebDriver, address: IPv4Address) -> None:
    subprocess.run(["probe-rs", "reset", "--chip", "esp32c3"], check=True)
    print("waiting for first measurement")
    time.sleep(1)
    assert mp_run.run("tests/client/mdns_resolver.py").wait() == 0, (
        "cresco did not rejoin network after reset"
    )
    dashboard = DashboardPage.open(driver, str(address))
    title = dashboard.get_title()
    assert title == "Cresco Dashboard", f"unexpected title: {title!r}"
    ec = dashboard.get_ec()
    assert ec != "Not calibrated", f"unexpected ec: {ec!r}"
    ph = dashboard.get_ph()
    assert ph != "Not calibrated", f"unexpected ph: {ph!r}"
    print(f"ec: {ec}")
    print(f"ph: {ph}")


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
    with run("tests/client/set_voltage.py") as runner:
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
        time.sleep(1)
        set_voltage(runner, "ec", 2.0)
        set_voltage(runner, "ph", 1.5)
        dashboard = calibrate_ph.finish()
        title = dashboard.get_title()
        assert title == "Cresco Dashboard", f"unexpected title: {title!r}"
        time.sleep(1)
        dashboard.refresh()
        ec = dashboard.get_ec()
        ph = dashboard.get_ph()
        assert is_close(ec, 5.0, 0.2), f"unexpected ec of: {ec!r}, should be 5.0"
        assert is_close(ph, 1.5, 0.2), f"unexpected ph of: {ph!r}, should be 1.5"
        print(f"ec: {ec} is close to actual 5.0")
        print(f"ph: {ph} is close to actual 1.5")
    assert runner.passed, "set_voltage.py did not pass"


def targets_test(args: argparse.Namespace) -> int:
    with Debugger() as debugger:
        targets = debugger.read("cross::blackboard::TARGETS")
    print(f"targets: {targets}")
    return 0 if targets.count("Equation::Constant{c: 5}") == 2 else 1


def mdns_normal_test(args: argparse.Namespace) -> int:
    return mp_run.run("tests/client/mdns_normal.py").wait()


def mdns_resolver_test(args: argparse.Namespace) -> int:
    return mp_run.run("tests/client/mdns_resolver.py").wait()


def mdns_legacy_test(args: argparse.Namespace) -> int:
    return mp_run.run("tests/client/mdns_legacy.py").wait()


def join_test(args: argparse.Namespace) -> int:
    return mp_run.run("tests/client/join.py").wait()


def dns_test(args: argparse.Namespace) -> int:
    return mp_run.run("tests/client/dns.py").wait()


TEST_FUNCTIONS: dict[str, Test] = {
    "storage": Test(run_web(storage_test), web_args),
    "dashboard_integrity": Test(run_web(dashboard_integrity_test), web_args),
    "targets": Test(targets_test),
    "mdns_normal": Test(mdns_normal_test),
    "mdns_resolver": Test(mdns_resolver_test),
    "mdns_legacy": Test(mdns_legacy_test),
    "join": Test(join_test),
    "dns": Test(dns_test),
}


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="test_name", required=True)
    for name, test in TEST_FUNCTIONS.items():
        test.args(sub.add_parser(name))
    args = parser.parse_args()
    return TEST_FUNCTIONS[args.test_name].run(args)


if __name__ == "__main__":
    sys.exit(main())
