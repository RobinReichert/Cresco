import argparse
import sys

from selenium import webdriver


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("ip", help="IP address of the device")
    args = parser.parse_args()

    options = webdriver.ChromeOptions()
    options.binary_location = "/usr/bin/chromium"
    options.add_argument("--headless=new")
    driver = webdriver.Chrome(options=options)
    try:
        driver.get(f"http://{args.ip}/")
        print(driver.title)
        return 0
    finally:
        driver.quit()


if __name__ == "__main__":
    sys.exit(main())
