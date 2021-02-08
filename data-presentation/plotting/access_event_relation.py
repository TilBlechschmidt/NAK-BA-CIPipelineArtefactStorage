import plotly.graph_objects as go
import plotly.io as pio
import sqlite3

dpi = 2

event_types = ["CI created", "CI finished", "Merge", "Access"]

con = sqlite3.connect("../data/out/simulation.db")

cur = con.cursor()
cur.execute("SELECT kind FROM SimulationEvent ORDER BY timestamp")
rows = cur.fetchall()

data = dict()
for event_type in range(0, len(event_types)):
    data[event_type] = dict(x=[], y=[], counter=0)

for (i, row) in enumerate(rows):
    kind = row[0]

    data[kind]['counter'] += 1
    data[kind]['x'].append(i)
    data[kind]['y'].append(data[kind]['counter'])

fig = go.Figure()

for (event_type, event_name) in enumerate(event_types):
    fig.add_trace(go.Scatter(x=data[event_type]['x'], y=data[event_type]['y'],
                        mode='lines',
                        name=event_name))

graph_boundary = len(rows) * 0.1
graph_range = (-graph_boundary, len(rows) + graph_boundary)
fig.update_layout(
    xaxis=dict(title="Total event count"),  # , range=graph_range),
    yaxis=dict(title="Cumulative events of type"),  # , range=graph_range),
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

# fig.show()
pio.orca.config.executable = "/usr/local/bin/orca"
fig.write_image("../data/out/plots/access-event-lineplot.pdf")
