import plotly.graph_objects as go
import plotly.io as pio
import csv
import numpy as np

dpi = 2
block_size = 1024 * 4

file_sizes = []
block_losses = []
with open("../data/example-pipeline-file-sizes.tsv") as csv_file:
    csv_reader = csv.reader(csv_file, delimiter='\t')
    rows = list(csv_reader)
    for row in rows:
        file_size = int(row[0])
        if file_size == 0:
            block_losses.append(0)
        else:
            block_losses.append(file_size % block_size)
        file_sizes.append(file_size)

print("Total pipeline size: " + str(sum(file_sizes)))
print("Total block losses: " + str(sum(block_losses)))
print("Loss median: " + str(np.median(block_losses)))

fig = go.Figure(data=[go.Histogram(x=block_losses, histnorm='probability', nbinsx=100)])

fig.update_layout(
    xaxis=dict(
        title="Block padding",
    ),
    yaxis=dict(
        title="Fraction of files in pipeline",
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
fig.write_image("../data/out/plots/block-alignment-histogram.pdf")
