from selenium.webdriver.common.by import By


def value_loaded(element_id: str):
    def condition(driver):
        text = driver.find_element(By.ID, element_id).text
        return text if text not in ("", "…") else False

    return condition
