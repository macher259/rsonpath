import os
import json

def concatenate_json_files(dir_path, output_path):
    with open(output_path, 'w') as outfile:
        outfile.write('{\n')
        first = True

        for filename in sorted(os.listdir(dir_path)):
            if not filename.endswith('.json'):
                continue
            key = filename[:-5]
            full_path = os.path.join(dir_path, filename)

            try:
                with open(full_path, 'r') as infile:
                    data = json.load(infile)
            except json.JSONDecodeError as e:
                print(f"Skipping {filename} due to JSON error: {e}")
                continue

            if not first:
                outfile.write(',\n')
            else:
                first = False

            json.dump(key, outfile)
            outfile.write(': ')
            json.dump(data, outfile)

        outfile.write('\n}\n')

if __name__ == "__main__":
    concatenate_json_files('/home/maciej/praca_exp/cord-19_2022-06-02/2022-06-02/document_parses/pdf_json',
                           'data/big/pdf_output.json')
    concatenate_json_files('/home/maciej/praca_exp/cord-19_2022-06-02/2022-06-02/document_parses/pmc_json',
                           'data/big/pmc_output.json')