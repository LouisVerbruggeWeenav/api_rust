from copy import deepcopy

def merge_json_objects(json_list):
    def deep_merge(a, b):
        if isinstance(a, dict) and isinstance(b, dict):
            result = deepcopy(a)
            for key in b:
                if key in result:
                    result[key] = deep_merge(result[key], b[key])
                else:
                    result[key] = deepcopy(b[key])
            return result
        elif isinstance(a, list) and isinstance(b, list):
            if all(isinstance(x, list) for x in a) and all(isinstance(x, list) for x in b) and len(a) == len(b):
                return [x + y for x, y in zip(a, b)]
            else:
                return a + b
        else:
            return deepcopy(b)

    def merge_list_of_dicts(list_of_dicts):
        merged = {}
        for d in list_of_dicts:
            for k, v in d.items():
                if k in merged:
                    merged[k] = deep_merge(merged[k], v)
                else:
                    merged[k] = deepcopy(v)
        return [{k: v} for k, v in merged.items()]

    merged_result = {}

    # Étape 1 : regrouper les blocs par clé principale ("GARMIN", "GARMIN 2", etc.)
    for item in json_list:
        for main_key, main_val in item.items():
            if main_key not in merged_result:
                merged_result[main_key] = []
            merged_result[main_key].extend(main_val)
    
    # Étape 2 : fusionner les commandes CAN (comme "OBC_DILONG_COMMAND ...") dans chaque bloc principal
    for main_key in merged_result:
        combined_entries = {}
        for entry in merged_result[main_key]:
            for command_key, command_val in entry.items():
                if command_key not in combined_entries:
                    combined_entries[command_key] = command_val
                else:
                    combined_entries[command_key] = deep_merge(combined_entries[command_key], command_val)

        # Étape 3 : fusionner les éléments CAN en double (comme MaxAllowableChargingVoltageL)
        for command_key in combined_entries:
            if isinstance(combined_entries[command_key], list):
                combined_entries[command_key] = merge_list_of_dicts(combined_entries[command_key])

        merged_result[main_key] = [{k: v} for k, v in combined_entries.items()]

    return [{k: v} for k, v in merged_result.items()]

