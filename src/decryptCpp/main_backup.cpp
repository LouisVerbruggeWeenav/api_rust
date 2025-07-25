// main.cpp
#include "dbcppp/include/dbcppp/Network.h"
#include <iostream>
#include <fstream>
#include <vector>
#include <typeinfo>  //for 'typeid' to work  
#include <stdio.h>

#include <string.h>

using namespace std;


// la premier fois au premier lancement:
// g++ -std=c++17 -I./dbcppp/include -g Untitled-1.cpp -L./build_dbcppp -ldbcppp -o compilationMain

constexpr uint32_t CAN_EXTENDED_MASK = 0x1FFFFFFF;

struct typeStructNode {
    std::string name;
    std::vector<std::string> signals;
};



std::vector<std::string> split(const std::string& str) {
    std::vector<std::string> tokens;
    std::istringstream iss(str);
    std::string token;
    
    while (iss >> token) {
        tokens.push_back(token);
    }
    
    return tokens;
}

vector<typeStructNode> traiterDBC(const string& path, vector<typeStructNode> data) {

    typeStructNode tempStruct;

    ifstream file(path);   
    if (!file.is_open()) {
        cerr << "Erreur ouverture DBC\n";
        return data;
    }

    
    unique_ptr<dbcppp::INetwork> network = dbcppp::INetwork::LoadDBCFromIs(file);
    if (!network) {
        cerr << "Erreur lecture réseau DBC\n";
        return data;
    }



    for (const auto& node : network->Nodes())
    {
        for (const auto& msg : network->Messages())
        {
            if (msg.Transmitter() == node.Name())
            {
                tempStruct.name = "0x" + to_string(msg.Id() & CAN_EXTENDED_MASK);
                //tempStruct.name = msg.Name();
                data.push_back(tempStruct);
            } 
        }
    }

    return data;
}


void decodeTram(
    uint32_t can_id,
    const std::unordered_map<uint32_t, const dbcppp::IMessage*>& messages_map,
    const std::vector<uint8_t>& raw_bytes
) {
    auto it = messages_map.find(can_id);
    if (it == messages_map.end()) {
        std::cout << "Message non trouvé pour ID " << std::hex << can_id << std::dec << "\n";
        return;
    }

    const auto* msg = it->second;
    std::cout << "== Message trouvé: " << msg->Name() << "\n";

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
            std::cout << sig.Name() << ": RAW=" << raw_val
                    << " → PHYS=" << phys_val
                    << " → ENUM='" << enum_text << "'\n";
        } else {
            std::cout << sig.Name() << ": RAW=" << raw_val
                    << " → PHYS=" << phys_val << "\n";
        }

    }
}




int main()
{


    vector<typeStructNode> data;

    data = traiterDBC("./WEENAV.dbc", data);

    for (const typeStructNode car: data) {

        std::cout << car.name << ", size signal: " << car.signals.size()<< endl;

    }


    std::string dbcPath = "WEENAV.dbc";  // Remplace avec ton fichier
    
    ifstream file(dbcPath);   

    unique_ptr<dbcppp::INetwork> network = dbcppp::INetwork::LoadDBCFromIs(file);

    std::unordered_map<uint32_t, const dbcppp::IMessage*> messages_map;

    for (const auto& msg : network->Messages()) {
        messages_map[msg.Id() & CAN_EXTENDED_MASK] = &msg;
    }


    std::ifstream fileTrc("./ok.trc");
    if (!fileTrc)
    {
        std::cerr << "Error : Can't open the file" << std::endl;
    }
    else
    {
        std::string line;
        while (std::getline(fileTrc, line))
        {
            auto words = split(line);

            for (const auto car: words) {
                std::cout << car << " | ";
            }

            int test;

            if (words.size() > 4) {
                if (words[0][0] != ';' && words[0] != "") {
                    string time =  words[1];
                    int idCan = stoi(words[3], 0, 16);
                    string lenght = words[4];
                    string dataLine = "";
                    
                    int len;
                    istringstream(lenght) >> len;
                    
                    for (int i = 0; i < len; i++) {
                        dataLine = dataLine + words[5+i];
                    }
                    std::istringstream iss(dataLine); // b'\x802\x00:\x02(\x00\x00' 419366912
                    std::string byte_str;
                    std::vector<uint8_t> raw_bytes;
                    while (iss >> byte_str)
                    {
                        raw_bytes.push_back(static_cast<uint8_t>(std::stoul(byte_str, nullptr, 16)));
                    }

                    std::cout << "data: " << dataLine << ", ID can: 0x" << idCan <<  endl;
                    decodeTram(idCan, messages_map, raw_bytes);
                    
                }
            }
            

            std::cout << "\n";


        }
    }

    int can_id = 0x18FF0800;  // Ton ID CAN étendu (en hex)

    // Ceci est la trame (payload) que tu veux décoder, en ordre de transmission (LSB -> MSB)
    std::string data_payload = "80 32 00 3A 02 28 00 00";

    std::istringstream iss(data_payload);
    std::string byte_str;
    std::vector<uint8_t> raw_bytes;

    while (iss >> byte_str)
    {
        raw_bytes.push_back(static_cast<uint8_t>(std::stoul(byte_str, nullptr, 16)));
    }

    // Vérif visuelle du buffer
    std::cout << "raw_bytes = [ ";
    for (uint8_t b : raw_bytes) {
        printf("0x%02X ", b);
    }
    std::cout << "]\n";

    // Décodage
    decodeTram(can_id, messages_map, raw_bytes);



    // fileTrc.close();

    return 0;
}