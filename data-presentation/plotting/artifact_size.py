import plotly.graph_objects as go
import plotly.io as pio
import json

with open("../data/out/artifacts.json") as file:
    raw_json = file.read()
    anonymised_json = raw_json
    with open("../data/anonymisation_strings.txt") as strings:
        for line in strings:
            a, b = line.strip().split("=")
            print("Replacing '" + a + "' with '" + b + "'")
            anonymised_json = anonymised_json.replace(a, b)
    data = json.loads(anonymised_json)

fig = go.Figure()

for entry in data:
    name = entry['name']
    x = entry['values']
    y = entry['keys']

    fig.add_trace(go.Box(
        x=x,
        y=y,
        name=name,
        boxpoints="suspectedoutliers",
        marker=dict(opacity=0.15)
    ))

dpi = 7

fig.update_layout(
    xaxis=dict(
        # range=[0, 1700 * 1000 * 1000],
        type="log",
        tickangle=90,
        side="top",
        exponentformat="SI",
        mirror=True,
        showgrid=True,
    ),
    yaxis=dict(
        zeroline=False,
        showgrid=True,
        ticks="outside",
        tickson="boundaries",
        ticklen=20
    ),
    boxmode='group',
    width=210 * dpi,
    height=260 * dpi  # 297 * dpi
)

fig.update_traces(orientation='h')
# fig.show()

pio.orca.config.executable = "/usr/local/bin/orca"
fig.write_image("../data/out/plots/artifacts-boxplot.pdf")
