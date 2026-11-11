import ipaddress

from selenium.webdriver.common.by import By
from selenium.webdriver.support import expected_conditions as condition
from selenium.webdriver.remote.webdriver import WebDriver

from tests.web.base_page import BasePage
from tests.web.settings import SettingsPage
from tests.web.helpers import value_loaded


class DashboardPage(BasePage):
    def __init__(self, driver: WebDriver, ip: ipaddress.IPv4Address):
        super().__init__(driver)
        driver.get(f"http://{str(ip)}/")

    def get_title(self) -> str:
        return self.driver.title

    def get_ph(self) -> str:
        return self.wait.until(value_loaded("ph"))

    def get_ec(self) -> str:
        return self.wait.until(value_loaded("ec"))

    def goto_settings(self) -> SettingsPage:
        self.wait.until(
            condition.element_to_be_clickable((By.ID, "settings-open"))
        ).click()
        return SettingsPage(self)
