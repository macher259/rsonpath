import requests
import gzip
from io import BytesIO
import os

def main():
    date_hour = "2021-06-06-18"
    url = f"https://data.gharchive.org/{date_hour}.json.gz"
    os.makedirs("data/github", exist_ok=True)
    output_file = f"data/github/{date_hour}.json"

    print(f"Downloading and converting: {url}")

    try:
        response = requests.get(url, stream=True)
        response.raise_for_status()
    except Exception as e:
        print(f"Failed to download {url}: {e}")
        return

    with open(output_file, "w", encoding="utf-8") as fout:
        fout.write("[\n")
        with gzip.GzipFile(fileobj=BytesIO(response.content)) as gz:
            first = True
            for line in gz:
                if not first:
                    fout.write(",\n")
                fout.write(line.decode("utf-8").rstrip())
                first = False
        fout.write("\n]\n")

    print(f"Saved: {output_file}")

if __name__ == "__main__":
    main()
