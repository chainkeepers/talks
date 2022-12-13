import pandas as pd


def main():
    py = pd.read_csv('log/python.log', names=['rtime', 'ctime', 'latency'])
    py = py.set_index('ctime')
    py = py['latency']
    print(py)

    def reader(filename):
        with open(filename) as infile:
            for line in infile:
                if '[' not in line and 'Resp' not in line:
                    yield map(float, line.split(', '))

    ru = pd.DataFrame(reader('log/rust.log'), columns=['rtime', 'ctime', 'latency'])
    ru = ru.set_index('ctime')
    ru = ru['latency']
    print(ru)
    
    pyru = pd.read_csv('log/rust-python.log', names=['rtime', 'ctime', 'rust_latency', 'latency'])
    pyru = pyru.dropna()
    pyru = pyru.set_index('ctime')
    pyru = pyru['latency']
    print(pyru)

    df = pd.merge(pd.merge(py, ru, on='ctime', suffixes=('_py', '_ru')), pyru, on='ctime', suffixes=('', '_pyru'))
    df.columns = ['py', 'ru', 'pyru']
    print(df)

    print(df.describe([0.01, 0.99]))


if __name__ == '__main__':
    main()

