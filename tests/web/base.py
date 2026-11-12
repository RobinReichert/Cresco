from collections.abc import Callable
from functools import wraps
from typing import Concatenate
from urllib.parse import urlparse

from selenium.webdriver.support import expected_conditions as condition
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.remote.webdriver import WebDriver

TIMEOUT = 30


class Component:
    def __init__(self, driver: WebDriver):
        self.driver = driver
        self.wait = WebDriverWait(driver, TIMEOUT)
        self._active = True

    def _click(self, by: str, value: str) -> None:
        self.wait.until(condition.element_to_be_clickable((by, value))).click()


def active[C: Component, **A, R](
    method: Callable[Concatenate[C, A], R],
) -> Callable[Concatenate[C, A], R]:
    """Only allow the call while the object is the one the browser is showing."""

    @wraps(method)
    def wrapper(self: C, *args: A.args, **kwargs: A.kwargs) -> R:
        if not self._active:
            raise RuntimeError(
                f"{type(self).__name__} is not active anymore, "
                "use the object returned by the last call that moved on"
            )
        return method(self, *args, **kwargs)

    return wrapper


class BasePage(Component):
    def __init__(self, driver: WebDriver, path: str):
        super().__init__(driver)
        self.path = path
        self.wait.until(lambda d: urlparse(d.current_url).path == path)

    @active
    def refresh(self) -> None:
        self.driver.refresh()

    @active
    def get_title(self) -> str:
        return self.driver.title
