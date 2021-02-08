cargo run --release -- batch 512 RAND FIFO LIFO MERGED-FIFO LRU-FIFO MRU-FIFO LF-FIFO SF-FIFO STATUS-FIFO SCORE
cargo run --release -- size-ramp 4 11 RAND FIFO LIFO MERGED-FIFO LRU-FIFO MRU-FIFO LF-FIFO SF-FIFO STATUS-FIFO SCORE
FALLBACK_DEBUG=1 cargo run --release -- batch 512 FIFO MRU-FIFO MRU.2-FIFO MRU.4-FIFO MRU.8-FIFO MRU.16-FIFO MRU.32-FIFO MRU.64-FIFO > ../data/out/simulation/mru_fallback_data.txt
SIZE_POPULATION_DUMP=1 cargo run --release -- one-shot 512 FIFO > ../data/out/simulation/size_data.txt

FALLBACK_DEBUG=1 cargo run --release -- one-shot 512 MERGED FIFO 2>/dev/null | tail -n 1
