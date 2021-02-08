import plotly.graph_objects as go
import plotly.io as pio

dpi = 2
input_path = "./manual/access-counts.csv"

data = []
with open(input_path) as strings:
    for line in strings:
        data.append(int(line.strip()))


out_of_bounds = 0
zero = 0
for sample in data:
    if sample > 209:
        out_of_bounds += 1
    elif sample == 0:
        zero += 1

print("Ignoring " + str(out_of_bounds) + " OOB values")
print(str(zero) + "/" + str(len(data)) + " samples are zero")

fig = go.Figure(data=[go.Histogram(x=data, histnorm='probability')])

fig.update_layout(
    xaxis=dict(
        title="Access count",
        range=(0, 210)
    ),
    yaxis=dict(
        title="Percentage of pipelines",
        tickformat=',.2%',
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

fig.show()
pio.orca.config.executable = "/usr/local/bin/orca"
fig.write_image("../data/out/plots/access-count-histogram.pdf")
