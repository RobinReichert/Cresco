from __future__ import annotations
from typing import TYPE_CHECKING

from tests.web.base_page import BasePage

if TYPE_CHECKING:
    from tests.web.dashboard import DashboardPage


class SettingsPage(BasePage):
    def __init__(self, dashboard: DashboardPage):
        super().__init__(dashboard.driver)

    def get_title(self) -> str:
        return self.driver.title
