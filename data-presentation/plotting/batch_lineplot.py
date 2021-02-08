import plotly.graph_objects as go
import plotly.io as pio
from os import walk, path
import csv
from math import log

dpi = 2
batch_directory = "../data/out/simulation/batch"


def generate_graph(output_path, algorithms=None, dot_first=0, strip_from_name=""):
    # if algorithms is None:
    #     _, _, algorithm_files = next(walk(batch_directory))
    #     algorithm_files.sort()
    #     # TODO Strip .csv postfixes

    fig = go.Figure()
    for algorithm in algorithms:
        with open(path.join(batch_directory, algorithm + ".csv")) as csv_file:
            csv_reader = csv.reader(csv_file, delimiter=',')
            x = []
            y = []
            for (i, row) in enumerate(csv_reader):
                if i == 0:
                    continue
                missed_fraction = float(row[-1])
                x.append(i)
                y.append(missed_fraction)

            fig.add_trace(go.Scatter(x=x, y=y,
                                line=dict(dash="dot") if dot_first > 0 else dict(),
                                mode='lines',
                                name=algorithm.replace(strip_from_name, "")))

        if dot_first > 0:
            dot_first -= 1

    fig.update_layout(
        xaxis=dict(title="Processed events"),
        yaxis=dict(
            title="Missed accesses",
            tickformat=',.2%',
            type="log",
            range=(log(0.1), log(1)),
            tickfont=dict(size=7)
        ),
        width=297 * dpi,
        height=210 * dpi,
        legend=dict(
            orientation="h",
            yanchor="bottom",
            y=1.02,
            xanchor="center",
            x=0.5
        ),
        margin=dict(
            l=0,
            r=0,
            b=0,
            t=0,
            pad=4
        ),
    )

    fig.write_image(output_path)
    # fig.show()


pio.orca.config.executable = "/usr/local/bin/orca"
generate_graph("../data/out/plots/lineplot-static.pdf", ["FIFO", "LIFO", "RAND"])
generate_graph("../data/out/plots/lineplot-layered.pdf", ["FIFO", "MRU-FIFO", "LRU-FIFO", "SF-FIFO", "LF-FIFO", "MERGED-FIFO", "STATUS-FIFO"], dot_first=1, strip_from_name="-FIFO")
generate_graph("../data/out/plots/lineplot-score.pdf", ["FIFO", "STATUS-FIFO", "MERGED-FIFO", "SCORE"], dot_first=3, strip_from_name="-FIFO")
generate_graph("../data/out/plots/lineplot-mru.pdf", ["FIFO", "MRU-FIFO", "MRU.2-FIFO", "MRU.4-FIFO", "MRU.8-FIFO", "MRU.16-FIFO", "MRU.32-FIFO", "MRU.64-FIFO"], dot_first=1, strip_from_name="-FIFO")
