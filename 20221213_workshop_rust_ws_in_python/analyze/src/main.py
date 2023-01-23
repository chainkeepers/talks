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

    pyws = pd.read_csv('log/python-websockets.log', names=['rtime', 'ctime', 'latency'])
    pyws = pyws.set_index('ctime')
    pyws = pyws['latency']
    print(pyws)

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

    pyruq = pd.read_csv('log/rust-python-quick.log', names=['rtime', 'ctime', 'rust_latency', 'latency'])
    pyruq = pyruq.dropna()
    pyruq = pyruq.set_index('ctime')
    pyruq = pyruq['latency']
    print(pyruq)

    df = pd.merge(pd.merge(pd.merge(pd.merge(py, pyws, on='ctime', suffixes=('_py', '_pyws')), ru, on='ctime', suffixes=('', '_ru')), pyru, on='ctime', suffixes=('', '_pyru')), pyruq, on='ctime', suffixes=('', '_pyruq'))
    df.columns = ['py', 'pyws', 'ru', 'pyru', 'pyruq']
    print(df)

    print(df.describe([0.01, 0.99]))


if __name__ == '__main__':
    main()
