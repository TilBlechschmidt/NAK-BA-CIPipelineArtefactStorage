import plotly.graph_objects as go
import plotly.io as pio

dpi = 2
input_path = "../data/out/simulation/size_data.txt"

data = []
with open(input_path) as strings:
    for line in strings:
        data.append(int(line.strip()))

fig = go.Figure(data=[go.Histogram(x=data)])

fig.update_layout(
    xaxis=dict(
        title="Pipeline size",
        exponentformat="SI",
    ),
    yaxis=dict(
        title="Pipeline count",
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
fig.write_image("../data/out/plots/pipeline-size-histogram.pdf")
