// main.cpp
#include "dbcppp/include/dbcppp/Network.h"
#include <iostream>
#include <fstream>
#include <vector>
#include <typeinfo>  //for 'typeid' to work  
#include <stdio.h>

#include <string.h>
#include <thread>  // pour sleep

#include <sstream>  // Pour std::istringstream



// pour le json
#include <nlohmann/json.hpp> 
using json = nlohmann::json;


using namespace std;

int jumpData = 1;   // 5 -> donc je reagrde que 4 ligne sur 5 dans le fichier trc
                    // 2 -> c'est une ligne sur 2
                    // 1 -> toute les lignes

constexpr uint32_t CAN_EXTENDED_MASK = 0x1FFFFFFF;

// Récupérer l'instant actuel
std::chrono::steady_clock::time_point getTimeNow() {
    return std::chrono::steady_clock::now();
}



// Calculer la différence en millisecondes
long long getTimeDifferenceMs(const std::chrono::steady_clock::time_point& t1,
                               const std::chrono::steady_clock::time_point& t2) {
    return std::chrono::duration_cast<std::chrono::milliseconds>(t2 - t1).count();
}


using Value = std::variant<std::string, double>;
using FastSignalMap = std::unordered_map<std::string, std::vector<std::vector<Value>>>;


struct typeStructNode {
    string name;
    FastSignalMap signals;
};

using typeDataStructData = std::unordered_map<std::string, std::vector<typeStructNode>>;

struct typeDataStruct {
    string name;
    vector<typeStructNode> data;
};



template <typename T>
int structInList(const std::vector<T>& listFindIndex, const std::string& nameFind) { 

    auto it = std::find_if(listFindIndex.begin(), listFindIndex.end(),
        [&nameFind](const T& p) { return p.name == nameFind; });

    if (it != listFindIndex.end()) {
        return static_cast<int>(std::distance(listFindIndex.begin(), it));
    }
    return -1;
}


std::vector<std::string> split(const std::string& str) {
    std::vector<std::string> tokens;
    std::istringstream iss(str);
    std::string token;
    
    while (iss >> token) {
        tokens.push_back(token);
    }
    
    return tokens;
}



json convertToJson(const typeDataStructData& dataStruct)
{
    json final_array = json::array();

    // Parcours des catégories, par ex. "BMS"
    for (const auto& [category_name, nodes] : dataStruct)
    {
        json category_json = json::array();

        // Parcours des noeuds dans la catégorie
        for (const auto& node : nodes)
        {
            json signals_json = json::array();

            // Parcours des signaux dans le noeud
            for (const auto& [signal_name, value_time_pairs] : node.signals)
            {
                json values = json::array();
                json timestamps = json::array();

                if (value_time_pairs.size() == 2) 
                {
                    const auto& vals = value_time_pairs[0];
                    const auto& times = value_time_pairs[1];

                    if (vals.size() == times.size())
                    {
                        for (size_t i = 0; i < vals.size(); ++i) {
                            const auto& val = vals[i];
                            const auto& ts = times[i];

                            // Conserver double ou string pour la valeur
                            if (std::holds_alternative<double>(val)) {
                                values.push_back(std::get<double>(val));
                            } else if (std::holds_alternative<std::string>(val)) {
                                values.push_back(std::get<std::string>(val));
                            }

                            // Pour le timestamp, on suppose une string (sinon convertit en string)
                            if (std::holds_alternative<std::string>(ts)) {
                                timestamps.push_back(std::get<std::string>(ts));
                            } else if (std::holds_alternative<double>(ts)) {
                                timestamps.push_back(std::to_string(std::get<double>(ts)));
                            }
                        }
                    }
                }

                json signal_json = json::array({ values, timestamps });

                signals_json.push_back({ { signal_name, signal_json } });
            }

            json node_json = { { node.name, signals_json } };

            category_json.push_back(node_json);
        }

        final_array.push_back({ { category_name, category_json } });
    }

    return final_array;
}











std::vector<uint8_t> parse_escaped_bytes(const std::string& escaped) {
    std::vector<uint8_t> result;

    auto start = escaped.find('\'');
    auto end   = escaped.rfind('\'');
    if (start == std::string::npos || end == std::string::npos || end <= start)
        return result;

    std::string hex = escaped.substr(start + 1, end - start - 1);

    for (size_t i = 0; i + 3 < hex.size(); i += 4) {
        if (hex[i] == '\\' && hex[i+1] == 'x') {
            std::string byte_str = hex.substr(i + 2, 2);
            uint8_t byte = static_cast<uint8_t>(std::stoi(byte_str, nullptr, 16));
            result.push_back(byte);
        }
    }

    return result;
}



vector<typeStructNode> traiterDBC(const unique_ptr<dbcppp::INetwork>& network, vector<typeStructNode> data) {
    for (const auto& msg : network->Messages()) {
        // Construire le nom basé sur l'ID du message
        std::string id_str = "0x" + to_string(msg.Id() & CAN_EXTENDED_MASK);

        // Vérifie si ce nom est déjà présent dans data
        bool deja_present = std::any_of(data.begin(), data.end(), [&](const auto& entry) {
            return entry.name == id_str;
        });

        if (!deja_present) {
            typeStructNode tempStruct;
            tempStruct.name = id_str;
            data.push_back(tempStruct);
        }
    }

    return data;
}




void addSignal(FastSignalMap& signalMap,
               const std::string& nameDecode,
               const std::string& valuesDecode,
               const std::string& dateTime)
{
    // Si le signal n'existe pas encore, on initialise avec 2 vecteurs vides
    if (signalMap.find(nameDecode) == signalMap.end()) {
        signalMap[nameDecode] = std::vector<std::vector<Value>>(2);
    }

    try {
        signalMap[nameDecode][0].push_back(std::stof(valuesDecode));
    } catch (...) {
        signalMap[nameDecode][0].push_back(valuesDecode);
    }

    signalMap[nameDecode][1].push_back(dateTime);
}



void decodeTram(
    uint32_t can_id,
    const std::unordered_map<uint32_t, const dbcppp::IMessage*>& messages_map,
    const std::vector<uint8_t>& raw_bytes,
    FastSignalMap& signalMap, 
    string dateTime) 
    {
    auto it = messages_map.find(can_id);
    if (it == messages_map.end()) return;

    const auto* msg = it->second;

    for (const auto& sig : msg->Signals()) {
        auto raw_val = sig.Decode(raw_bytes.data());
        double phys_val = sig.RawToPhys(raw_val);

        bool found_enum = false;
        std::string enum_text;

        for (const auto& val_encoding : sig.ValueEncodingDescriptions()) {
            if (val_encoding.Value() == raw_val) {
                enum_text = val_encoding.Description();
                found_enum = true;
                break;
            }
        }

        if (found_enum) {
            addSignal(signalMap, sig.Name(), enum_text, dateTime);
        } else {
            addSignal(signalMap, sig.Name(), std::to_string(phys_val), dateTime);
        }
    }
}

void cleanStruct(const unique_ptr<dbcppp::INetwork>& network, vector<typeStructNode> &data, typeDataStructData &dataStruct) 
{



    //for (typeStructNode node : data) {
    for (int i = data.size() - 1; i >= 0; i--) {
        if (data[i].signals.empty()) {
            data.erase(data.begin() + i);
        }
    }

    // maintenant je vais mettre tout les données dans leur catégorie, 
    // dans GARMIN / OBC_MTA / Unknown...
  

    std::vector<std::string> all_nodes;
    for (const auto& node : network->Nodes()) {
        all_nodes.push_back(node.Name());
    }
    all_nodes.push_back("Unknown");



    for (typeStructNode signal: data) {

        for (const auto& msg : network->Messages()) { 


            if ( signal.name == "0x" + to_string(msg.Id() & CAN_EXTENDED_MASK) ) {

                stringstream ss;
                ss << hex << ( msg.Id() & CAN_EXTENDED_MASK);

                signal.name = msg.Name() + " (ID: 0x" + ss.str() +")";

                if (std::find(all_nodes.begin(), all_nodes.end(), msg.Transmitter()) == all_nodes.end()) {
                    dataStruct["Unknown"].push_back(signal);
                } else {
                    dataStruct[msg.Transmitter()].push_back(signal);
                }
            }
        }
    }    
}






extern "C" const typeDataStructData decrypt_cpp(json tram_can_json)
{

    vector<typeStructNode> data;


    typeDataStructData dataStruct;

    typeStructNode tempStruct;


    //ifstream file("./src/decryptCpp/WEENAV.dbc");   
    ifstream file("./src/decryptCpp/WEENAV.dbc");   

    unique_ptr<dbcppp::INetwork> network = dbcppp::INetwork::LoadDBCFromIs(file);
    if (!file.is_open()) {
        cerr << "Erreur ouverture DBC\n";
    }    
    
    else if (!network) {
        cerr << "Erreur lecture réseau DBC\n";
        
    }

    else {
        data = traiterDBC(network, data);
    }



    std::unordered_map<uint32_t, const dbcppp::IMessage*> messages_map;

    std::unordered_map<std::string, int> indexCache;


    for (const auto& msg : network->Messages()) {

        messages_map[msg.Id() & CAN_EXTENDED_MASK] = &msg;
    }

    int numberLine = 0;

    auto start = getTimeNow();

    // std::ifstream fileTrc("./src/decryptCpp/ok.trc");
    std::ifstream fileTrc("./src/decryptCpp/ok.trc");

    if (!fileTrc)
    {
        std::cerr << "Error : Can't open the file" << std::endl;
    }
    else
    {
        for (const auto& line : tram_can_json) {
                    
            unsigned long idCan = line["id"].get<unsigned long>();

            std::ostringstream oss;
            oss << "0x" << std::hex << idCan;
            std::string idCanHexStr = oss.str();


            string timee = line["timestamp"].get<std::string>();

            std::string message_str = line["message"].get<std::string>();


            std::vector<uint8_t> raw_bytes = parse_escaped_bytes(message_str);


            int index = structInList(data, "0x"+to_string(idCan));
            if (index >= 0) {
                
                decodeTram(idCan, messages_map, raw_bytes, data[index].signals, timee);

            } else {            
                // cout << "il y a un ID inconnu..." ;
            }
            
        }
    }


    auto end = getTimeNow();

    long long diffMs = getTimeDifferenceMs(start, end);

    cleanStruct(network, data, dataStruct);
    diffMs = getTimeDifferenceMs(start, end);

    return dataStruct;
}


int main() {

    ostringstream ss;
    ss << cin.rdbuf();
    string input = ss.str();

    if (input.empty()) {
        cerr << "Entrée vide sur stdin" << endl;
        return 1;
    }

    try {
        const typeDataStructData result = decrypt_cpp(json::parse(input));

        json json_result = convertToJson(result);

        std::cout << json_result.dump(4); // dump indenté pour lisibilité


    } catch (const exception& e) {
        cerr << "Erreur lors du décryptage : " << e.what() << endl;
        return 1;
    }



    return 0;
}