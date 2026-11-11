from __future__ import annotations


from selenium.webdriver.common.by import By
from selenium.webdriver.remote.webdriver import WebDriver

from tests.web.base import BasePage, Component, active
import tests.web.calibrate
from tests.web.helpers import value_loaded


class DashboardPage(BasePage):
    def __init__(self, driver: WebDriver):
        super().__init__(driver, "/")

    @classmethod
    def open(cls, driver: WebDriver, host: str) -> DashboardPage:
        driver.get(f"http://{host}")
        return cls(driver)

    @active
    def get_title(self) -> str:
        return self.driver.title

    @active
    def get_ph(self) -> str:
        return self.wait.until(value_loaded("ph"))

    @active
    def get_ec(self) -> str:
        return self.wait.until(value_loaded("ec"))

    @active
    def open_settings(self) -> SettingsDrawer:
        self._click(By.ID, "settings-open")
        self._active = False
        return SettingsDrawer(self)


class SettingsDrawer(Component):
    def __init__(self, dashboard: DashboardPage):
        super().__init__(dashboard.driver)

    @active
    def close_settings(self) -> DashboardPage:
        self._click(By.ID, "settings-close")
        self.wait.until(
            lambda d: (
                d.find_element(By.ID, "settings-drawer").get_attribute("aria-hidden")
                == "true"
            )
        )
        self._active = False
        return DashboardPage(self.driver)

    @active
    def goto_calibrate_ph(self) -> tests.web.calibrate.CalibratePhPage:
        self._click(By.CSS_SELECTOR, '#settings-drawer a[href="/calibrate/ph"]')
        self._active = False
        return tests.web.calibrate.CalibratePhPage(self.driver)

    @active
    def goto_calibrate_ec(self) -> tests.web.calibrate.CalibrateEcPage:
        self._click(By.CSS_SELECTOR, '#settings-drawer a[href="/calibrate/ec"]')
        self._active = False
        return tests.web.calibrate.CalibrateEcPage(self.driver)
