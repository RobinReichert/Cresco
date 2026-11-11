import argparse
from ipaddress import IPv4Address
import sys
import shutil
from typing import Callable

from selenium import webdriver
from selenium.webdriver.remote.webdriver import WebDriver

from tests.web.dashboard import DashboardPage

PAGE_LOAD_TIMEOUT = 30

type TestFunc = Callable[[WebDriver, IPv4Address], int]


def dashboard_integrity_test(driver: WebDriver, address: IPv4Address) -> int:
    dashboard = DashboardPage(driver, address)
    title = dashboard.get_title()
    assert title == "Cresco Dashboard", f"unexpected title: {title!r}"
    return 0


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
        return TEST_FUNCTIONS[args.test_name](driver, args.ip)
    except Exception as e:
        print(e)
    finally:
        driver.quit()
    return 1


if __name__ == "__main__":
    err = main()
    print("FAIL" if err else "PASS")
    sys.exit(err)
