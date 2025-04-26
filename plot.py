import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv('static/results.csv')

methods = [
    ('load_time', 'Load Time'),
    ('search_time', 'Search Time'),
    ('add_time', 'Add Record Time'),
    ('update_time', 'Update Record Time'),
    ('search_after_update_time', 'Search After Update Time')
]

colors = {
    'small.dat': 'blue',
    'medium.dat': 'green',
    'large.dat': 'red',
}

for method_col, method_label in methods:
    plt.figure(figsize=(10,6))

    for file in df['file'].unique():
        subset = df[df['file'] == file]
        plt.plot(subset['t'], subset[method_col], marker='o', label=f'{file}', color=colors.get(file, None))

    plt.title(f'Benchmark - {method_label}')
    plt.xlabel('t (degree of B-Tree)')
    plt.ylabel('Time (seconds)')
    plt.grid(True)
    plt.legend()
    plt.xticks(subset['t'])
    plt.savefig(f'static/{method_col}_benchmark.png')
    plt.show()

