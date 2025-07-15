
from datetime import datetime
import ast
import time


class FileTrc:
    def __init__(self, tramCan):
        self.data = tramCan

        self.start_trc = None
        self.start_day_trc = None
        self.idManquant = set()


    def load_trc(self):
        """Load the TRC file and return its content."""
        try:
            with open(self.file_path, 'r') as file:
                data = file.readlines()
            self.data = data
        except Exception as e:
            raise Exception(f"Error loading TRC file: {e}")
        
    def findMessageById(self, dataStruct, id: str):
        """Find the index of a message by its ID in the data structure."""
        for indexData, elem in enumerate(dataStruct):
            for key in elem:
                for indexElem, item in enumerate(elem[key]):
                    if id in list(item.keys())[0]:
                        return (indexData, list(elem.keys())[0], indexElem, list(item.keys())[0])
        return None
    
    def decodeMessage(self, can_id, dataMessage, dataDbc, timee):

        # dt = datetime.datetime.fromtimestamp(timestamp)
        # print(dt.strftime("%d/%m/%Y à %H:%M:%S.%f")[:-3])

        message = dataDbc.decode_message(can_id, dataMessage)  
        temp = [{elem: [[message[elem]], [timee]]} for elem in message]
        return temp
            

    def cleanData(self, dataStruct):
        """Clean the data structure by removing empty lists."""
        if not dataStruct:
            raise Exception("No data structure provided. Please provide a valid data structure.")
        
        decalageNode = 0
        for indexData in range(len(dataStruct)):
            decalage = 0
            elem = dataStruct[indexData-decalageNode]
            for key in elem:
                for i in range(len(elem[key])):
                    item = elem[key][i-decalage]
                    if not item[list(item.keys())[0]]:
                        del dataStruct[indexData-decalageNode][key][i-decalage]
                        decalage += 1
            if not dataStruct[indexData-decalageNode][key]:
                del dataStruct[indexData-decalageNode]
                decalageNode += 1


        return dataStruct


    def find_data(self, dataStruct: list, dataDbc: object):
        """Extract data from the TRC file."""

        if not self.data:
            raise Exception("No data loaded. Please load the TRC file first.")
        if dataStruct is None or not isinstance(dataStruct, list):
            raise Exception("Invalid data structure provided. Please provide a valid list of data structures.")

        
        start_time = time.time()


        for line in self.data:
            
            try:
                timee = line['timestamp']
                can_id = line['id']
                dlc = line['length']
                data = ast.literal_eval(line['message'])
                index = self.findMessageById(dataStruct, f'{can_id:X}')


                if dataStruct[index[0]][index[1]][index[2]][index[3]] == []:
                    dataStruct[index[0]][index[1]][index[2]][index[3]] = self.decodeMessage(can_id, data, dataDbc, timee)
                    
                else:
                    if isinstance(dataStruct[index[0]][index[1]][index[2]][index[3]], list):
                        
                        for elem in self.decodeMessage(can_id, data, dataDbc, timee):
                            tempIndex = next(((tempIndex, tempElem, list(elem.keys())[0])  for tempIndex, tempElem in enumerate(dataStruct[index[0]][index[1]][index[2]][index[3]]) if list(elem.keys())[0] in tempElem), None)
                        
                            if tempIndex is not None:

                                dataStruct[index[0]][index[1]][index[2]][index[3]][tempIndex[0]][list(elem.keys())[0]][0].append(elem[list(elem.keys())[0]][0][0]) # time
                                dataStruct[index[0]][index[1]][index[2]][index[3]][tempIndex[0]][list(elem.keys())[0]][1].append(elem[list(elem.keys())[0]][1][0]) # data

                            else:
                                raise Exception(f"Erreur: la clé {list(elem.keys())[0]} existe déjà à l'index {tempIndex[0]} dans la structure de données.")

                    else:
                        print(f"Erreur: la structure de données à l'index {index} n'est pas une liste.")

            except Exception:
                #print(f"Erreur lors de l'extraction des données de la ligne: {line}")
                self.idManquant.add(can_id)
                continue

        print(f"temps du traitement: {time.time() - start_time}")

        dataStruct = self.cleanData(dataStruct)
        return dataStruct
    
    def getIdManquant(self):
        """Return the IDs that were not found in the data structure."""
        return self.idManquant