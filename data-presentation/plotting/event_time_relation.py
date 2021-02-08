import plotly.graph_objects as go
import plotly.io as pio
import sqlite3
from datetime import datetime, timedelta

dpi = 2
cutoff_timestamp = 1608049887
cutoff_date = datetime.fromtimestamp(cutoff_timestamp)

con = sqlite3.connect("../data/out/simulation.db")

cur = con.cursor()
cur.execute("SELECT timestamp FROM SimulationEvent ORDER BY timestamp")
rows = cur.fetchall()

x_values = list(map(lambda r: cutoff_date + timedelta(seconds=r[0]), rows))
y_values = list(range(0, len(rows)))
days = (x_values[-1] - x_values[0]).days

fig = go.Figure()

fig.add_trace(go.Scatter(x=x_values, y=y_values,
                    mode='lines',
                    name='lines'))

fig.update_layout(
    xaxis=dict(nticks=days,tickangle=90),
    yaxis=dict(title="Cumulative event count"),
    width=297 * dpi,
    height=210 * dpi,
    margin=dict(
        l=0,
        r=0,
        b=0,
        t=0,
        pad=4
    )
)

# fig.show()
pio.orca.config.executable = "/usr/local/bin/orca"
fig.write_image("../data/out/plots/event-time-lineplot.pdf")
