
import os
import re
import sys
import time
import base64
import importlib.util
from pathlib import Path
import typing
from urllib.parse import urljoin
from dataclasses import dataclass, fields
from typing import Annotated, Any, TypeVar, Union, get_args, get_origin, get_type_hints


import lxml.html
import requests
from selenium import webdriver
# import undetected_chromedriver as uc
from selenium.webdriver.common.by import By
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.support.ui import WebDriverWait
from selenium.common.exceptions import NoSuchElementException

from library import Link, Text, Time, Url

T = TypeVar('T')

class ScraperBrowser:
    def __init__(self, timeout = 20, simple = False, **kwargs):
        self.timeout = timeout
        self.simple = simple
        options = Options()

        if self.simple:
            # be really nice about only requesting what you need
            prefs = {
                "profile.managed_default_content_settings.images": 2,  # Block images
                "profile.default_content_setting_values.stylesheets": 2, # Block CSS
                "profile.default_content_setting_values.javascript": 2, # Block JavaScript
                "profile.default_content_setting_values.notifications": 2, # Block notifications
                "profile.default_content_setting_values.popups": 2, # Block popups
                "profile.default_content_setting_values.plugins": 2, # Block plugins
                "profile.default_content_setting_values.geolocation": 2, # Block geolocation
                "profile.default_content_setting_values.media_stream": 2, # Block media access
            }
            options.add_experimental_option("prefs", prefs)


        options.add_argument(f'--browser-version=137')
        if False:
            self.driver = uc.Chrome(version_main=137, options=options)
        else:
            self.driver = webdriver.Chrome(options=options)

        self.wait = WebDriverWait(self.driver, timeout, kwargs)

    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.driver.quit()
    
    def nagivate_to(self, url: str):
        self.driver.get(url)


    def wait_seconds(self, seconds: int):
        time.sleep(seconds)

    def wait_for_page(self, url_pattern):
        if not self.simple:
            def url_matches_pattern(driver):
                current_url = driver.current_url
                return re.search(url_pattern, current_url) is not None
            self.wait.until(url_matches_pattern)
        else:
            print(f"Warning: In simple=True mode, wait_for_page only checks the last known URL")
    
    def load_from_file(self, file_path: str):
        if os.path.exists(file_path):
            abs_file_path = os.path.abspath(file_path)
            file_path = f"file://{abs_file_path}"
        self.driver.get(file_path)

    def get_cookies(self):
        return self.driver.get_cookies()
     
    def dump_cookies(self):
        cookies = self.get_cookies()
        with open('cookies.txt', 'w') as f:
            for c in cookies:
                f.write(f"{c['name']}={c['value']}\n")
                
    def _get_element(self, xpath, context):
        return context.find_element(By.XPATH, xpath)

    def _get_elements(self, xpath, context):
        return context.find_elements(By.XPATH, xpath)
        
    def parse(self, model_class: T, __context_element = None) -> T:
        if not isinstance(model_class, type):
            raise RuntimeError(f"'{model_class}' is not an uninstantiated class.")
    
        if __context_element is None:
            __context_element = self.driver.find_element(By.TAG_NAME, "html")

        parsed_data = {}

        type_annotations = get_type_hints(model_class, include_extras=True)
        for field_name, annotated_type in type_annotations.items():
            if get_origin(annotated_type) is not Annotated:
                raise RuntimeError(f"Field '{field_name}' in '{model_class.__name__}' is not correctly annotated with typing.Annotated.")
            
            base_type, xpath = get_args(annotated_type)
            
            parsed_data[field_name] = self._parse_field(xpath, base_type, __context_element)
        return self._create_instance(model_class, parsed_data)

    def _parse_field(self, xpath: str, field_type: Any, context_element: Any) -> Any:
        # Optional[T]
        if get_origin(field_type) is typing.Union:
            args = get_args(field_type)
            if len(args) == 2 and args[1] == type(None):
                actual_type = next(arg for arg in args if arg is not type(None))
                try:
                    return self._parse_field(xpath, actual_type, context_element)
                except (NoSuchElementException, IndexError, RuntimeError):
                    return None
        # List[T]
        if get_origin(field_type) is list:
            elements_list = self._get_elements(xpath, context_element)
            actual_type = get_args(field_type)[0]
            results = []
            for element in elements_list:
                results.append(self._parse_field('.', actual_type, element))
            return results
        # Class
        if hasattr(field_type, '__annotations__') and not str(field_type.__module__).startswith("builtins") and not str(field_type.__module__).startswith("library"):
            nested_context = self._get_element(xpath, context_element)
            return self.parse(field_type, nested_context)
        
        element = self._get_element(xpath, context_element)
        # Str
        if field_type == str or field_type is Text:
            return element.text
        # Link
        if field_type is Link:
            for attr in ['href', 'src', 'data-url']:
                url = element.get_attribute(attr)
                if url:
                    return url
            return None
        # Time
        if field_type is Time:
            content = element.text
            return parse_string_time_to_text(content)
        # Url
        if field_type is Url:
            return self.driver.current_url
        # Integer
        if field_type == int:
            text_content = element.text
            numbers = re.findall(r'\d+', str(text_content).replace(',', '').replace(' ', ''))
            return int(numbers[0]) if numbers else 0
        # idk
        raise TypeError(f"Unsupported field type: {field_type} for XPath: {xpath}")

    def _create_instance(self, model_class, parsed_data):
        class DynamicModel:
            _is_dynamic_model_instance = True

            def __init__(self, **kwargs):
                self.__data = kwargs
                setattr(self, '__name__', model_class.__name__)
                for key, value in kwargs.items():
                    setattr(self, key, value)

            def __repr__(self):
                attrs = ', '.join(f'{k}={v!r}' for k, v in self.__dict__.items())
                return f'{model_class.__name__}({attrs})'
                       
            def _set_kv(self, k, v):
                self.__data[k] = v
                setattr(self, k, v)
                
            def json(self):
                json_data = {}
                for key, value in self.__data.items():
                    if isinstance(value, DynamicModel):
                        json_data[key] = value.json()
                    elif isinstance(value, list):
                        json_data[key] = [item.json() if hasattr(item, '_is_dynamic_model_instance') else item for item in value]
                    else:
                        json_data[key] = value
                return json_data

        DynamicModel.__name__ = model_class.__name__
        DynamicModel.__qualname__ = model_class.__qualname__
        
        return DynamicModel(**parsed_data)

def parse_string_time_to_text(time_string):
    if not time_string:
        return 0

    normalized_time_str = time_string.lower().replace("ago", "").strip()

    unit_multipliers = {
        's': 1, 'sec': 1, 'second': 1, 'seconds': 1,
        'm': 60, 'min': 60, 'minute': 60, 'minutes': 60,
        'h': 3600, 'hr': 3600, 'hour': 3600, 'hours': 3600,
        'd': 86400, 'day': 86400, 'days': 86400,
        'w': 604800, 'wk': 604800, 'week': 604800, 'weeks': 604800, # 7 days
        'y': 31536000, 'yr': 31536000, 'year': 31536000, 'years': 31536000 # 365 days
    }

    total_seconds = 0

    
    pattern = re.compile(
        r'(\d+)\s*'
        r'('
        r's(?:ec(?:ond)?s?)?|'  # s, sec, second, seconds
        r'm(?:in(?:ute)?s?)?|'  # m, min, minute, minutes
        r'h(?:r|our)?s?|'       # h, hr, hour, hours
        r'd(?:ay)?s?|'          # d, day, days
        r'w(?:k|eek)?s?|'       # w, wk, week, weeks
        r'y(?:r|ear)?s?'        # y, yr, year, years
        r')'
    )

    matches = pattern.findall(normalized_time_str)

    if not matches and normalized_time_str:
        raise ValueError(f"Invalid time string format or unrecognized units: '{time_string}'")

    for value_str, unit_str in matches:
        try:
            value = int(value_str)
        except ValueError:
            raise ValueError(f"Could not parse number '{value_str}' in '{time_string}'")

        multiplier = unit_multipliers.get(unit_str)
        if multiplier is None:
            raise ValueError(f"Unrecognized time unit: '{unit_str}' in '{time_string}'")
        
        total_seconds += value * multiplier

    return total_seconds
