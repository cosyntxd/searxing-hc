import json
from collections import Counter

# todo: relative
with open("/Users/ryan/Github/Hackclub-projects/complete_database.json") as f:
    data = json.load(f)

counter = Counter()

def collect_values(obj, parent_key=None):
    if isinstance(obj, dict):
        for k, v in obj.items():
            if k in ["extra", "updates", "description"]:
                continue
            collect_values(v, k)
    elif isinstance(obj, list):
        for item in obj:
            collect_values(item, parent_key)
    else:
        if not isinstance(obj, int):
            if obj:
                for word in obj.split(" "):
                    counter[word.lower()] += 1

collect_values(data)

for value, count in counter.most_common(33):
    print(f"{value}:{count}")
