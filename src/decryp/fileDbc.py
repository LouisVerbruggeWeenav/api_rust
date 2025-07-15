

import cantools
import pprint

class FileDbc:
    def __init__(self, file):
        self.file = file
        self.titles = []
        self.underTitles = []
        self.data = None
        self.dataStruct = []

        self.load_dbc()
        self.initDataStruct()

    def load_dbc(self):
        """Charge le fichier DBC"""
        try:
            self.data = cantools.database.load_file(self.file)
        except Exception as e:
            raise Exception(f"Erreur lors du chargement du DBC: {e}")


    def initDataStruct(self):
        """il faut ouvrir le fichier dbc et ouvrir recup le titre dans la cat message"""
        if not self.data:
            raise Exception("No data loaded. Please load the DBC file first.")       
        
        for title in [node.name for node in self.data.nodes] + ["Unknown"]:
            self.dataStruct.append({title: [{f"{tx.name} (ID: 0x{tx.frame_id:X})": []} for tx in [msg for msg in self.data.messages if title in msg.senders or title == "Unknown"]]})
        print("je suis dans le fichier fileDbc.py, dans la fonction initDataStruct")

    def getDataStruct(self):
        """Retourne la structure de données"""
        return self.dataStruct

    def getData(self):
        """Retourne les données"""
        return self.data
    

