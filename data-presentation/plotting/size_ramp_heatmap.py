import plotly.graph_objects as go
import plotly.io as pio
from os import walk, path
import re
import csv

size_ramp_directory = "../data/out/simulation/size-ramp"
dpi = 2


def sorted_nicely(l):
    """ Sort the given iterable in the way that humans expect."""
    convert = lambda text: int(text) if text.isdigit() else text
    alphanum_key = lambda key: [convert(c) for c in re.split('([0-9]+)', key)]
    l.sort(key=alphanum_key)


_, size_directories, _ = next(walk(size_ramp_directory))
sorted_nicely(size_directories)


def read_missed_percentage(file_path):
    with open(file_path) as csv_file:
        csv_reader = csv.reader(csv_file, delimiter=',')
        rows = list(csv_reader)
        missed_fraction = float(rows[-1][-1])
        return missed_fraction


# We just naively *assume* that every size directory contains all algorithms
algorithm_names = []
size_names = size_directories
z_data = []
for size in size_directories:
    # Read the algorithm files from the directory
    dirname, _, statistics_files = next(walk(path.join(size_ramp_directory, size)))

    # Establish an order for the algorithms on the first run
    if len(algorithm_names) == 0:
        data = list(map(lambda filename: (filename, read_missed_percentage(path.join(dirname, filename))), statistics_files))
        data.sort(key=lambda i: i[1])
        algorithm_names = list(map(lambda i: i[0][:-4], data))

    # Extract the last value for each algorithm
    data = []
    for algorithm_name in algorithm_names:
        data.append(read_missed_percentage(path.join(dirname, algorithm_name + ".csv")))

    z_data.append(data)

fig = go.Figure(data=go.Heatmap(
                   z=z_data,
                   x=list(map(lambda name: name.replace("-FIFO", ""), algorithm_names)),
                   y=size_names,
                   colorscale="inferno",
                   colorbar=dict(tickformat=',.0%')))

fig.update_layout(
    xaxis=dict(title="Algorithm"),
    yaxis=dict(title="Simulated disk size"),
    width=297 * dpi,
    height=210 * dpi,
    margin=dict(
        l=0,
        r=0,
        b=0,
        t=0,
        pad=0
    ),
)

fig.show()
pio.orca.config.executable = "/usr/local/bin/orca"
fig.write_image("../data/out/plots/size-ramp-heatmap.pdf")
