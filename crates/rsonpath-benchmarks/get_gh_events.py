import requests
import gzip
import json
from io import BytesIO
import os

def stream_hour_to_output(fout, date: str, hour: int, is_first: bool):
    key = f"{date}-{hour}"
    url = f"https://data.gharchive.org/{key}.json.gz"
    print(f"Processing: {url}")

    try:
        response = requests.get(url, stream=True)
        response.raise_for_status()
    except Exception as e:
        print(f"Failed to download {url}: {e}")
        return False

    if not is_first:
        fout.write(",\n")
    fout.write(json.dumps(key))
    fout.write(": [")

    with gzip.GzipFile(fileobj=BytesIO(response.content)) as gz:
        first = True
        for line in gz:
            if not first:
                fout.write(",\n")
            fout.write(line.decode("utf-8").rstrip())
            first = False

    fout.write("]")
    return True


def main():
    date = "2024-01-01"
    output_file = "data/big/gharchive-2015-01-01.json"
    os.makedirs("data/big", exist_ok=True)

with open(output_file, "w", encoding="utf-8") as fout:
        fout.write("{\n")

        for hour in range(24):
            success = stream_hour_to_output(fout, date, hour, is_first=(hour == 0))
            if not success:
                continue

        fout.write("\n}\n")

    print(f"Saved: {output_file}")

if __name__ == "__main__":
    main()
