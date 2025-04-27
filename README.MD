# RustDB

✨ A simple and efficient **persistent database** using a **B-tree** index.

Supports **insertion**, **updating**, and **searching** records stored in a binary datafile.

## ✨ Features

- Insert records into a binary file
- Update existing records by key
- Search for records using an efficient B-tree index
- Benchmarking support for different B-tree orders (`t`)
- Plot performance results using Python

## 📦 How to Run

### 1. Clone the Repository

```bash
git clone https://github.com/AndyStan112/rustdb.git
cd rustdb
```

### 2. Prepare Benchmark Files

Place your test data files (`small.dat`, `medium.dat`, `large.dat`) in the `static/` directory.

### 3. Run the Benchmark

```bash
cargo run
```

This will:
- Run the benchmark on different B-tree configurations
- Save results to `static/results.csv`

### 4. Plot Results

After generating the benchmark results:

```bash
pip install matplotlib
python plot.py
```

Multiple graphs will be generated showing the performance comparison.

## 🛠️ Code Structure

| Module  | Description  |
|:--------|:--------------|
| `mod btree` | B-tree index implementation over a binary file |
| `mod table` | Table abstraction to manage records and their B-tree index |
| `benchmark` | Code to measure load, search, add, update timings |

## 📈 Benchmark Metrics

Each benchmark collects:
- **Load Time** – Time to open and index the datafile
- **Search Time** – Time to find an existing key
- **Add Time** – Time to insert a new key and record
- **Update Time** – Time to modify an existing record
- **Search After Update Time** – Time to find the updated record

Results are saved in `static/results.csv` for plotting.

## 🧪 Sample Benchmark Output

```bash
File          t    Load(s)    Search(s)  Add(s)      Update(s)   Search2(s)
small.dat     2    0.003864   0.000030    0.000384    0.000055    0.000044
small.dat     4    0.004913   0.000040    0.000281    0.000049    0.000041
small.dat     8    0.004949   0.000064    0.000344    0.000056    0.000048
small.dat    16    0.007261   0.000060    0.000328    0.000050    0.000044
medium.dat    2    1.601475   0.000028    0.000217    0.000030    0.000026
medium.dat    4    1.295197   0.000039    0.000227    0.000034    0.000031
medium.dat    8    1.820784   0.000047    0.000311    0.000045    0.000041
medium.dat   16    2.769343   0.000075    0.000459    0.000075    0.000062
large.dat     2  344.357414   0.002623    0.000511    0.000074    0.000072
large.dat     4  326.195499   0.000061    0.000450    0.000061    0.000057
large.dat     8  451.851824   0.000086    0.000773    0.000114    0.000120
large.dat    16  671.499069   0.003259    0.000837    0.000107    0.000101
```

(*Measured on my machine. Real times may vary depending on hardware and system load.*)
