
import sys
import os
import pprint

current_dir = os.path.dirname(os.path.abspath(__file__))
if current_dir not in sys.path:
    sys.path.insert(0, current_dir)

from src.decryp.fileDbc import FileDbc
from src.decryp.fileTrc import FileTrc
from src.decryp.joinJson import merge_json_objects

import json

def decryp(tramCan):

    fileDbc = FileDbc("./src/decryp/WEENAV.dbc")
    fileTrc = FileTrc(json.loads(tramCan))

    allData = fileTrc.find_data(fileDbc.getDataStruct(), fileDbc.getData())  # Extract data from the TRC file at initialization

    idManquants = fileTrc.getIdManquant()
    if len(idManquants) > 0:
        print(f"\033[91mAttention, {len(idManquants)} ID manquants dans le fichier DBC, veuillez vérifier le fichier DBC.\033[0m")
        print("ID manquants dans le fichier DBC:")
        print([f"{elem:X}" for elem in fileTrc.getIdManquant()])

    return json.dumps(allData, default=str)


def concatJson(listPath):

    with open(listPath[0]) as f:
        dataFirstFile = json.load(f)
    
    for i in range(len(listPath)-1):

        with open(listPath[i+1]) as f:
            dataSecondFile = json.load(f)

        print("1")
        dataFirstFile = merge_json_objects(dataFirstFile + dataSecondFile)

    return json.dumps(dataFirstFile)