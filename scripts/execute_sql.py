import argparse
import itertools as itt
import requests

parser = argparse.ArgumentParser()

parser.add_argument("file", help="SQL file")
parser.add_argument("-u", "--url", dest="url", default="http://127.0.0.1:26650", help="Host URL")
parser.add_argument("-s", "--start-sequence", dest="sequence", type=int, default=0, help="Start sequence")
parser.add_argument("-b", "--batch-size", dest="batch", type=int, default=10, help="Batch size")
parser.add_argument("-g", "--gas-limit", dest="gas", type=int, default=10000000000000, help="Gas limit")

args = parser.parse_args()

with open(args.file) as file:
    lines = file.readlines()
    batches = [[e[1] for e in grp] for k, grp in itt.groupby(enumerate(lines), key=lambda x: x[0] // args.batch % 2)]

    for i, b in enumerate(batches):
        b = [s.rstrip() for s in b]
        stmts = ''.join(map(str, itt.chain.from_iterable(b)))
        seq = args.sequence + i
        x = requests.post(args.url + "/v1/execute", json={'stmts': stmts,
                                                          'sequence': seq,
                                                          'gas_limit': args.gas})
        print(x.json())
        print(seq)
