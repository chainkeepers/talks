import sys

import pandas as pd


def parse_line(line):
    try:
        time_str, data_str = line.split(':', 1)
        loc_time, ex_time, latency = time_str.split(', ')
        data = data_str.split()

        record = {
            'loc_time': float(loc_time),
            'ex_time': float(ex_time),
            'latency': float(latency),
            'data0': float(data[0]),
            'data1': float(data[1]),
            'data2': float(data[2]),
            'data3': float(data[3]),
        }

    except:
        record = {}

    return record


def load_log(filename):
    with open(filename) as infile:
        records = (parse_line(line) for line in infile)
        df = pd.DataFrame(records)

    df = df.dropna()
    df = df[df['ex_time'] > 0]
    df = df[df['loc_time'] > 0]

    return df.copy()


def describe(x):

    q = [.025, .25, .5, .75, .975]
    report = x.describe(q)
    x_trunc = x[x.between(report.loc['25%'], report['75%'])]
    x_trunc2 = x[x.between(report.loc['2.5%'], report['97.5%'])]

    return pd.DataFrame({
        'all': report,
        'center 95%': x_trunc2.describe(q),
        'center 50%': x_trunc.describe(q),
    })

    
def main(rustfile, pythonfile):
    rust_df = load_log(rustfile)
    python_df = load_log(pythonfile)
    
    print("Report")
    print("Times in ms.")
    print('Latency diffs are "Python - Rust".')
    print('Positive latency diff means Rust is faster.')

    print()
    print("Rust latency")
    print(describe(rust_df['latency']))

    print()
    print("Python latency")
    print(describe(python_df['latency']))

    # 1)

    # match origination timestamps

    assert not rust_df['ex_time'].duplicated().any()
    assert not python_df['ex_time'].duplicated().any()

    df = pd.merge(rust_df, python_df, on='ex_time')
    df2 = pd.merge(rust_df, python_df, on=['ex_time', 'data0', 'data1', 'data2', 'data3'], suffixes=('_rst', '_py'))

    for i in range(4):
        assert df[f'data{i}_x'].equals(df[f'data{i}_y']), f'data{i} _x and _y not equal'
        assert df[f'data{i}_x'].equals(df2[f'data{i}']), f'data{i} _x and df2 not equal'
        assert df[f'data{i}_y'].equals(df2[f'data{i}']), f'data{i} _y and df2 not equal'

    df2['py_rst_latency'] = 1000 * (df2['loc_time_py'] - df2['loc_time_rst'])

    # latency at all data

    print()
    print("Latency diffs")

    print()
    print("All")
    print(describe(df2['py_rst_latency']))

    # latency at all changes

    is_changed = df2[['data0', 'data1', 'data2', 'data3']].diff() != 0
    print()
    print("Any change")
    print(describe(df2.loc[is_changed.any(axis=1), 'py_rst_latency']))

    # filter changes in data

    is_changed_much = (
        (df2[['data0', 'data3']].pct_change().abs() > 5)
        | (df2[['data1', 'data2']].pct_change().abs() > 0.5)
    )
    print()
    print("Big change")
    print(describe(df2.loc[is_changed_much.any(axis=1), 'py_rst_latency']))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])

    # The results are optimized
    # The results are not using pyston

