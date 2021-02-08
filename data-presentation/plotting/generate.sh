#!/bin/sh
source ./venv/bin/activate

python3 access_count_distribution.py
python3 access_event_relation.py
python3 artifact_size.py
python3 batch_lineplot.py
python3 event_time_relation.py
python3 mru_fallback.py
python3 pipeline_size_histogram.py
python3 size_ramp_heatmap.py
python3 tabular_pipeline_counts.py
python3 ml_datagen.py > ../data/out/ml_data.csv
python3 file_counts_histogram.py
python3 file_block_alignment.py

deactivate
