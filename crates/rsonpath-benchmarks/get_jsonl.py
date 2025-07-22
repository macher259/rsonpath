#!/usr/bin/env python3
import json
import sys
import os

def usage():
    prog = os.path.basename(sys.argv[0])
    print(f"Usage: {prog} <input.jsonl> [output.json]")
    sys.exit(1)

def main():
    if not (2 <= len(sys.argv) <= 3):
        usage()

    in_path = sys.argv[1]
    out_path = sys.argv[2] if len(sys.argv) == 3 else None

    try:
        with open(in_path, 'r', encoding='utf-8') as f:
            data = [json.loads(line) for line in f if line.strip()]
    except Exception as e:
        print(f"Error reading {in_path}: {e}", file=sys.stderr)
        sys.exit(1)

    output = json.dumps(data, indent=2, ensure_ascii=False)

    if out_path:
        try:
            with open(out_path, 'w', encoding='utf-8') as f:
                f.write(output)
        except Exception as e:
            print(f"Error writing {out_path}: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        print(output)

if __name__ == "__main__":
    main()
