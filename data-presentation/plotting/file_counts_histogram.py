import plotly.graph_objects as go
import plotly.io as pio
import csv

dpi = 2
file_counts = []
with open("../data/raw/file_counts.csv") as csv_file:
    csv_reader = csv.reader(csv_file, delimiter=' ')
    rows = list(csv_reader)
    for row in rows:
        file_count = int(row[1])
        file_counts.append(file_count)

print("Number of file count data points: " + str(len(file_counts)))

fig = go.Figure(data=[go.Histogram(x=file_counts, histnorm='probability', nbinsx=50)])

fig.update_layout(
    xaxis=dict(
        title="File count",
        exponentformat="SI",
    ),
    yaxis=dict(
        title="Fraction of observed pipelines",
        tickformat=',.2%'
    ),
    width=297 * dpi,
    height=210 * dpi,
    margin=dict(
        l=0,
        r=0,
        b=0,
        t=0,
        pad=4
    ),
)

# fig.show()
pio.orca.config.executable = "/usr/local/bin/orca"
fig.write_image("../data/out/plots/pipeline-file-count-histogram.pdf")
