

import json

def run(name):
    print(f"hello PYTHON FUNCTION !!!")
    nameLoad = json.loads(name)
    for elem in nameLoad:
        print(elem['timestamp'])
    return json.dumps([1, 2, 3, 4, 5])