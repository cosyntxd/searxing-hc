import re
import json
from collections import Counter
from pathlib import Path

# todo: relative
with open("/Users/ryan/Github/Hackclub-projects/complete_database.json") as f:
    data = json.load(f)

counter = Counter()

def collect_values(obj, parent_key=None):
    if isinstance(obj, dict):
        for k, v in obj.items():
            if k in ["extra", "author", "main_image"]:
                continue
            collect_values(v, k)
    elif isinstance(obj, list):
        for item in obj:
            collect_values(item, parent_key)
    else:
        if not isinstance(obj, int):
            if obj:
                if not "//" in obj:
                    cleaned = re.sub(r"[^A-Za-z0-9\s]", "", obj)
                    for word in cleaned.split(" "):
                        if not any(c.isdigit() for c in word):
                            if word:
                                counter[word.lower()] += 1

collect_values(data)

out_file = Path(__file__).parent.parent / "data" / "word_list.txt"
with open(out_file, "w") as f:
    for value, count in counter.most_common(20_000):
        f.write(f"{value}\n")
