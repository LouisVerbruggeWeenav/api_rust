
import sys
import os

current_dir = os.path.dirname(os.path.abspath(__file__))
if current_dir not in sys.path:
    sys.path.insert(0, current_dir)

from src.decryp.fileDbc import FileDbc
from src.decryp.fileTrc import FileTrc

import json
import pprint

def decryp(tramCan):

    print("OUVERTURE DU FICHIER .DBC ")
    fileDbc = FileDbc("./src/decryp/WEENAV.dbc")
        
    print("OUVERTURE DU FICHIER .TRC ")
    fileTrc = FileTrc(json.loads(tramCan))

    print("GO LE DECODAGE !")
    allData = fileTrc.find_data(fileDbc.getDataStruct(), fileDbc.getData())  # Extract data from the TRC file at initialization

    idManquants = fileTrc.getIdManquant()
    if len(idManquants) > 0:
        print(f"\033[91mAttention, {len(idManquants)} ID manquants dans le fichier DBC, veuillez vérifier le fichier DBC.\033[0m")
        print("ID manquants dans le fichier DBC:")
        print([f"{elem:X}" for elem in fileTrc.getIdManquant()])

    pprint.pprint(allData)

    return json.dumps(allData)

print("HELLO DEPUIS LE FICHIER DECREYPT.PY")