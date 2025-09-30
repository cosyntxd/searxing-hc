import json
from typing import Annotated, Optional, Union, get_args, get_origin, get_type_hints
from library import *
from library import Text, Link
from browser import ScraperBrowser
from events import *
import math
import concurrent.futures
import traceback

NUM_THREADS = 4
MAX_ID = 16000 

def worker(start_inclusive: int, end_exclusive: int, thread_idx: int):
    print(f"[T{thread_idx}] starting: {start_inclusive}..{end_exclusive - 1}")
    try:
        scraper_factory = dyn_import_scraper("summer", 2025)

        with ScraperBrowser(simple=True) as browser:
            for i in range(start_inclusive, end_exclusive):
                try:
                    url = f"https://summer.hackclub.com/projects/{i}"
                    browser.nagivate_to(url)
                    result = browser.parse(scraper_factory())

                    scraped_data_dict = {result.__name__: result.json()}

                    response = send_scraped_data_to_backend(
                        url=BACKEND_URL,
                        secret=AUTH_SECRET,
                        scraped_data=scraped_data_dict
                    )
                    print(f"[T{thread_idx}] {i} -> {response.status_code} {response.text}")
                except Exception as e:
                    pass
    except Exception as e:
        print(f"[T{thread_idx}] fatal error starting worker: {e}")
        traceback.print_exc()
    finally:
        print(f"[T{thread_idx}] finished.")

def chunk_ranges(total: int, chunks: int):
    chunk_size = math.ceil(total / chunks)
    ranges = []
    for k in range(chunks):
        start = k * chunk_size
        end = min(start + chunk_size, total)
        if start < end:
            ranges.append((start, end))
    return ranges

if __name__ == "__main__":
    BACKEND_URL = "http://searxing.hackclub.app"
    AUTH_SECRET = "not_a_secret_secret"

    ranges = chunk_ranges(MAX_ID, NUM_THREADS)

    with concurrent.futures.ThreadPoolExecutor(max_workers=NUM_THREADS) as ex:
        futures = []
        for idx, (s, e) in enumerate(ranges):
            futures.append(ex.submit(worker, s, e, idx))
        for fut in concurrent.futures.as_completed(futures):
            try:
                fut.result()
            except Exception as e:
                print("worker crashed:", e)
