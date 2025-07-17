import pprint
import json

from src.decryp.joinJson import merge_json_objects

def concatJson(listPath):

    with open(listPath[0]) as f:
        dataFirstFile = json.load(f)
    
    for i in range(len(listPath)-1):

        with open(listPath[i+1]) as f:
            dataSecondFile = json.load(f)

        print("1")
        dataFirstFile = merge_json_objects(dataFirstFile + dataSecondFile)

    return dataFirstFile

data = concatJson(["boats/Boat_1/2025-07-15-12_28.json", "boats/Boat_1/2025-07-15-12_29.json", "boats/Boat_1/2025-07-15-12_30.json"])

print(json.dumps(data))