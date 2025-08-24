import base64
import importlib
import json
import os
from pathlib import Path
from typing import Callable, Optional
import requests

def dyn_import_scraper(event: str, year: int) -> Callable:
    path = Path() / "scraper" / "events" / f"{year}-{event}.py"
    if not path.exists():
        raise FileNotFoundError(f"{path} does not exist.")
    
    module_name = f"___internal_mod_{event}_{year}"

    spec = importlib.util.spec_from_file_location(module_name, str(path))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    if not hasattr(module, "run"):
        raise AttributeError(f"No `run` function found in {path}")

    return module.run

def send_scraped_data_to_backend(url, secret, scraped_data) -> requests.Response:
    endpoint = f"{url}/add"
    payload = {
        "secret": secret,
        "data": json.dumps(scraped_data)
    }
    headers = {"Content-Type": "application/json"}
    response = requests.post(endpoint, json=payload, headers=headers)
    response.raise_for_status()
    return response

# typing
class Link(str):
    """String type for text content."""
    pass
class Text(str):
    """String type for URLs/links."""
    pass
class Time(str):
    """String type for geniuses of urls."""
    pass
class Url(str):
    """String type for time"""
    pass