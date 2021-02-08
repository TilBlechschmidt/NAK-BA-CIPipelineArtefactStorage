import plotly.graph_objects as go
import plotly.io as pio

dpi = 2
algorithms = ['MRU', 'MRU-2', 'MRU-4', 'MRU-8', 'MRU-16', 'MRU-32', 'MRU-64']
values = [
    1533.0 / 2593.0,
    1611.0 / 2604.0,
    1816.0 / 2627.0,
    2152.0 / 2651.0,
    2441.0 / 2666.0,
    2593.0 / 2668.0,
    2656.0 / 2669.0
]

fig = go.Figure([go.Bar(x=algorithms, y=values)])

fig.update_layout(
    xaxis=dict(title="Algorithm"),
    yaxis=dict(
        title="Fallback ratio",
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
fig.write_image("../data/out/plots/mru-fallback.pdf")
