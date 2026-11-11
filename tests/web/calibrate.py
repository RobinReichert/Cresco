from __future__ import annotations

from selenium.webdriver.common.by import By
from selenium.webdriver.support import expected_conditions as condition
from selenium.webdriver.remote.webdriver import WebDriver

from tests.web.base import BasePage, active
import tests.web.dashboard


class CalibrationError(Exception):
    """The calibration page showed an error message."""


class CalibratePage(BasePage):
    @active
    def get_step(self) -> str:
        return self.driver.find_element(By.ID, "step-label").text

    @active
    def start(self) -> None:
        self._advance()

    @active
    def submit_point(self, actual: float) -> None:
        field = self.wait.until(
            condition.visibility_of_element_located((By.ID, "actual"))
        )
        field.clear()
        field.send_keys(str(actual))
        self._advance()

    @active
    def finish(self) -> tests.web.dashboard.DashboardPage:
        step = self.get_step()
        if step != "Done":
            raise CalibrationError(f"cannot finish, calibration is at {step!r}")
        self._click(By.ID, "next")
        self._active = False
        return tests.web.dashboard.DashboardPage(self.driver)

    def _error(self) -> str:
        return self.driver.find_element(By.ID, "error").text

    def _advance(self) -> None:
        """Click the button and wait until the page shows the next step or an error."""
        before = self.get_step()
        self._click(By.ID, "next")
        self.wait.until(lambda _: self._error() or self.get_step() != before)
        if error := self._error():
            raise CalibrationError(error)


class CalibratePhPage(CalibratePage):
    def __init__(self, driver: WebDriver):
        super().__init__(driver, "/calibrate/ph")


class CalibrateEcPage(CalibratePage):
    def __init__(self, driver: WebDriver):
        super().__init__(driver, "/calibrate/ec")
