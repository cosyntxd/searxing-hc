import os
import re
import time
from pathlib import Path
from typing import Annotated, Any, TypeVar, Union, get_args, get_origin, get_type_hints
from urllib.parse import urljoin

import lxml.html
import requests
from selenium.common.exceptions import NoSuchElementException

from library import Link, Text, Time, Url

T = TypeVar('T')


class ScraperBrowser:
    def __init__(self, timeout=20, simple=True, **kwargs):
        self.timeout = timeout
        self.simple = simple
        
        self.driver = None
        self.wait = None
        self.session = None
        self.html_tree = None
        self.current_url = None

        if self.simple:
            self.session = requests.Session()
            self.session.headers.update({
                'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
            })
        else:
            from selenium import webdriver
            from selenium.webdriver.chrome.options import Options
            from selenium.webdriver.chrome.service import Service
            from selenium.webdriver.common.by import By
            from selenium.webdriver.support.ui import WebDriverWait
            from webdriver_manager.chrome import ChromeDriverManager
            from selenium.webdriver.support import expected_conditions as EC


            
            DRIVER_PATH = ChromeDriverManager().install()
            options = Options()
            options.add_argument("--headless=new")
            options.add_argument("--disable-gpu")
            
            self.driver = webdriver.Chrome(
                service=Service(DRIVER_PATH), 
                options=options
            )

            self.driver.execute_cdp_cmd("Network.enable", {})
            self.driver.execute_cdp_cmd("Network.setBlockedURLs", {
                "urls": ["*.png", "*.jpg", "*.jpeg", "*.gif", "*.webp", "*.svg", "*.mp4", "*.webm"]
            })
            self.wait = WebDriverWait(self.driver, timeout, **kwargs)

    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.driver:
            self.driver.quit()
        if self.session:
            self.session.close()
    
    def nagivate_to(self, url: str):
        if self.simple:
            response = self.session.get(url, timeout=self.timeout)
            response.raise_for_status()
            self.html_tree = lxml.html.fromstring(response.content)
            self.current_url = response.url
        else:
            self.driver.get(url)

    def wait_seconds(self, seconds: int):
        time.sleep(seconds)

    def wait_for_page(self, url_pattern):
        if not self.simple:
            self.wait.until(EC.url_matches(url_pattern))
        else:
            print(f"Warning: In simple=True mode, no-op")
    
    def load_from_file(self, file_path: str):
        if self.simple:
            path = Path(file_path).resolve()
            content = path.read_text(encoding='utf-8')
            self.html_tree = lxml.html.fromstring(content)
            self.current_url = path.as_uri()
        else:
            abs_file_path = os.path.abspath(file_path)
            file_uri = f"file://{abs_file_path}"
            self.driver.get(file_uri)

    def get_cookies(self):
        if self.simple:
            return [{'name': c.name, 'value': c.value} for c in self.session.cookies]
        else:
            return self.driver.get_cookies()
     
    def dump_cookies(self, file_path='cookies.txt'):
        cookies = self.get_cookies()
        with open(file_path, 'w') as f:
            for c in cookies:
                f.write(f"{c['name']}={c['value']}\n")
                
    def _get_element(self, xpath, context):
        if self.simple:
            elements = context.xpath(xpath)
            if not elements:
                raise NoSuchElementException(f"no element found for XPath: {xpath}")
            return elements[0]
        else:
            return context.find_element(By.XPATH, xpath)

    def _get_elements(self, xpath, context):
        if self.simple:
            return context.xpath(xpath)
        else:
            return context.find_elements(By.XPATH, xpath)
        
    def parse(self, model_class: T, __context_element = None) -> T:
        if not isinstance(model_class, type):
            raise RuntimeError(f"'{model_class}' is not an uninstantiated class??")
    
        if __context_element is None:
            if self.simple:
                if self.html_tree is None:
                    raise RuntimeError("No page loaded")
                __context_element = self.html_tree
            else:
                __context_element = self.driver.find_element(By.TAG_NAME, "html")

        parsed_data = {}
        type_annotations = get_type_hints(model_class, include_extras=True)

        for field_name, annotated_type in type_annotations.items():
            if get_origin(annotated_type) is not Annotated:
                raise RuntimeError(f"field '{field_name}' in '{model_class.__name__}' is not correctly annotated.")
            
            base_type, xpath = get_args(annotated_type)
            parsed_data[field_name] = self._parse_field(xpath, base_type, __context_element)
        
        return self._create_instance(model_class, parsed_data)

    def _parse_field(self, xpath: str, field_type: Any, context_element: Any) -> Any:
        # Optional[T]
        if get_origin(field_type) is Union:
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
            return [self._parse_field('.', actual_type, element) for element in elements_list]
        # Nested Class
        is_custom_class = hasattr(field_type, '__annotations__') and not str(field_type.__module__).startswith(("builtins", "library"))
        if is_custom_class:
            nested_context = self._get_element(xpath, context_element)
            return self.parse(field_type, nested_context)
        
        element = self._get_element(xpath, context_element)
        
        if self.simple:
            if field_type in (str, Text):
                return element.text_content().strip()
            if field_type is Link:
                for attr in ['href', 'src', 'data-url']:
                    if (url := element.get(attr)):
                        return urljoin(self.current_url, url)
                return None
            if field_type is Time:
                return parse_string_time_to_text(element.text_content())
            if field_type is Url:
                return self.current_url
            if field_type is int:
                text_content = element.text_content()
                numbers = re.findall(r'\d+', str(text_content).replace(',', '').replace(' ', ''))
                return int(numbers[0]) if numbers else 0
        else: # Selenium mode
            if field_type in (str, Text):
                return element.text
            if field_type is Link:
                for attr in ['href', 'src', 'data-url']:
                    if (url := element.get_attribute(attr)):
                        return url
                return None
            if field_type is Time:
                return parse_string_time_to_text(element.text)
            if field_type is Url:
                return self.driver.current_url
            if field_type is int:
                text_content = element.text
                numbers = re.findall(r'\d+', str(text_content).replace(',', '').replace(' ', ''))
                return int(numbers[0]) if numbers else 0
        
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
                attrs = ', '.join(f'{k}={v!r}' for k, v in self.__data.items())
                return f'{model_class.__name__}({attrs})'
                       
            def _set_kv(self, k, v):
                self.__data[k] = v
                setattr(self, k, v)
                
            def json(self):
                json_data = {}
                for key, value in self.__data.items():
                    if hasattr(value, '_is_dynamic_model_instance'):
                        json_data[key] = value.json()
                    elif isinstance(value, list):
                        json_data[key] = [item.json() if hasattr(item, '_is_dynamic_model_instance') else item for item in value]
                    else:
                        json_data[key] = value
                return json_data

        DynamicModel.__name__ = model_class.__name__
        DynamicModel.__qualname__ = model_class.__qualname__
        
        return DynamicModel(**parsed_data)

def parse_string_time_to_text(time_string: str) -> int:
    if not time_string:
        return 0

    normalized_time_str = time_string.lower().replace("ago", "").strip()
    unit_multipliers = {
        's': 1, 'sec': 1, 'second': 1,
        'm': 60, 'min': 60, 'minute': 60,
        'h': 3600, 'hr': 3600, 'hour': 3600,
        'd': 86400, 'day': 86400,
        'w': 604800, 'wk': 604800, 'week': 604800,
        'y': 31536000, 'yr': 31536000, 'year': 31536000
    }

    # plural
    for unit in list(unit_multipliers.keys()):
        if len(unit) > 1:
            unit_multipliers[unit + 's'] = unit_multipliers[unit]
            
    pattern = re.compile(r'(\d+)\s*([a-z]+)')
    matches = pattern.findall(normalized_time_str)
    
    if not matches:
        return 0

    total_seconds = 0
    for value_str, unit_str in matches:
        value = int(value_str)
        multiplier = unit_multipliers.get(unit_str)
        if multiplier:
            total_seconds += value * multiplier
            
    return total_seconds