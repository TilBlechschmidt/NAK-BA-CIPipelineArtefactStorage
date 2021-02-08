import csv
from os import walk, path

batch_directory = "../data/out/simulation/size-ramp/512GB"


def read_data(file_path):
    with open(file_path) as csv_file:
        csv_reader = csv.reader(csv_file, delimiter=',')
        rows = list(csv_reader)
        stored_pipeline_counts = []

        for row in rows[1:]:
            stored_pipeline_counts.append(int(row[1]))

        delete_count = rows[-1][2]
        avg_stored = float(sum(stored_pipeline_counts)) / float(len(stored_pipeline_counts))
        return avg_stored, delete_count


_, _, statistics_files = next(walk(batch_directory))

for file in statistics_files:
    name = file[:-4]
    avg_stored, delete_count = read_data(path.join(batch_directory, file))
    print(name + "," + str(round(avg_stored)) + "," + delete_count)
