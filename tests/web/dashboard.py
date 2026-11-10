import argparse
import shutil
import sys

from selenium import webdriver


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("ip", help="IP address of the device")
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
    driver.set_page_load_timeout(30)
    try:
        driver.get(f"http://{args.ip}/")
        print(driver.title)
        return 0
    finally:
        driver.quit()


if __name__ == "__main__":
    sys.exit(main())
