


use actix_web::{get, post, web, App, HttpServer, Responder, web::Json};
use serde_json;

use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::fs;

use std::ffi::CString;

use std::sync::Mutex;
use serde_json::Value;
use serde::{Serialize, Deserialize};


// Ensure the database module is declared

mod database;
use crate::database::{boat::Boat, connection::Connection};
use std::cell::RefCell;



struct AppState {
    boat: Mutex<Boat>,
}


#[derive(Deserialize)]
struct InfoFrontOne {
    id: i32,
}

#[derive(Deserialize, Clone)]
struct StructInfoBoat {
    name: String,
    startRecord: String
}


#[derive(Deserialize, Clone)]
struct InfoRaspberrypi{
    infoBoat: StructInfoBoat,
    structData: String,
}

#[derive(Deserialize, Clone)]
struct InfoFrontByName {
    name: String,
}

#[derive(Debug, Deserialize)]

struct CanFrame {
    timestamp: String,
    id: u32,
    lenght: String,
    message: String
}


#[post("/raspberrypi/data")]
async fn raspberryData(data: web::Data<AppState>, info: web::Json<InfoRaspberrypi>) ->  impl Responder {

    let data_struct: Value = functionDecryptPython(info.structData.clone()).expect("msg");
    let mut boat = data.boat.lock().unwrap();

    boat.add_boat(info.infoBoat.name.clone(), info.infoBoat.startRecord.clone(), data_struct);

    "OK"
}


#[get("/")]
async fn index() -> impl Responder {

    println!("ppppppppppppp");

    let nom_fichier = "../boats/test/test.json";

    let contenu = fs::read_to_string(nom_fichier).expect("Quelque chose s'est mal passé lors de la lecture du fichier");
    let json: serde_json::Value = serde_json::from_str(&contenu).expect("msg");

    Json(
        json
    )
}




#[get("/boats/grouped")]
async fn get_grouped_boats(data: web::Data<AppState>) -> impl Responder {


    let mut json: serde_json::Value = serde_json::Value::Null;
    let mut boat = data.boat.lock().unwrap();

    json = match boat.get_grouped_boats() {
        
        Ok(groupBoats) => {
            match serde_json::to_value(&groupBoats) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("Erreur de sérialisation: {}", e);
                    serde_json::json!({ "error": format!("Erreur de sérialisation: {}", e) })
                }

            }
        }

        Err(e) => {
            eprintln!("Erreur GroupBy : {}", e);
            serde_json::json!({ "error": format!("Parsing GroupBy: {}", e) })
        }

    };

    Json(json)

}


#[post("/boats/by-name")]
async fn get_boat_by_id_post(data: web::Data<AppState>, info: web::Json<InfoFrontByName>) -> impl Responder {

    let mut json: serde_json::Value = serde_json::Value::Null;
    let mut boat = data.boat.lock().unwrap();

    json = match boat.get_boat_by_name(info.name.clone()) {
        
        Ok(groupBoats) => {
            match serde_json::to_value(&groupBoats) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("Erreur de sérialisation: {}", e);
                    serde_json::json!({ "error": format!("Erreur de sérialisation: {}", e) })
                }
            }
        }

        Err(e) => {
            eprintln!("Erreur SQL byName : {}", e);
            serde_json::json!({ "error": format!("Parsing GroupBy: {}", e) })
        }

    };

    Json(json)


}
    




#[post("/boats/one")]
async fn get_boat_one(data: web::Data<AppState>, info: web::Json<InfoFrontOne>) -> impl Responder {
    
    let mut json: serde_json::Value = serde_json::Value::Null;

    let mut boat = data.boat.lock().unwrap();
    match boat.get_boat_by_id(info.id) {
        Ok(boats) => {           

            json = match fs::read_to_string(format!("./boats/{}/{}.json", boats.name, boats.path)) {
                Ok(contenu) => { 
                    match serde_json::from_str(&contenu) {
                        Ok(dataJson) => dataJson,
                        
                        Err(e) => {
                            eprintln!("Erreur de parsing JSON : {}", e);
                            serde_json::json!({ "error": format!("Parsing JSON failed: {}", e) })
                        }
                    }
                },
                Err(e) => {
                    let pathFail = format!("./boats/{}/{}.json", boats.name, boats.path);
                    eprintln!("Erreur de lecture du fichier : {} {}", e, pathFail);
                    serde_json::json!({ "error": format!("File read failed: {}", e) })
                }
            }
        },
        Err(e) => {
            eprintln!("Erreur lors de la récupération : {}", e);
            json = serde_json::json!({ "error": format!("{}", e) });
            
        }
    }

    Json(json)
}






fn functionDecryptPython(tram_can: String) -> Result<Value, Box<dyn std::error::Error>> {
    // Lire le fichier Python
    let code_str = fs::read_to_string("./src/decryp/decryp.py")
        .expect("Fichier Python introuvable");

    let code_cstring = CString::new(code_str).expect("CString::new failed");
    let filename = CString::new("decryp.py").unwrap();
    let modulename = CString::new("main").unwrap();

    println!("ok dans la fonction dec");

    let parsed = RefCell::new(Value::Null);

    Python::with_gil(|py| {
        let result = (|| -> PyResult<()> {
            let module = PyModule::from_code(
                py,
                code_cstring.as_c_str(),
                filename.as_c_str(),
                modulename.as_c_str(),
            )?;

            let result = module.getattr("decryp")?.call1((tram_can, ))?;
            let json_str: String = result.extract()?;
            let value: Value = serde_json::from_str(&json_str).expect("JSON invalide");

            // Affichage utile pour debug
            println!("Résultat JSON : {}", value);

            *parsed.borrow_mut() = value;
            Ok(())
        })();

        if let Err(e) = result {
            e.print(py); // Affiche traceback Python
        }
    });

    Ok(parsed.into_inner())
}



#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
// fn main() {
    let mut database = Connection::new("localhost".to_string(), 3306, "root".to_string(), "welcome1".to_string(), "boat_directory".to_string());
    database.connect();

    // get all data boats:
    let mut boat = Boat::new(database.getConn());


    // GLOBAL_DATA.with(|text| {
    //     println!("Global string is {}", *text.borrow());
    // });



    // GET ALL:
    //
    // match boat.get_all_boats(){
    //     Ok(boats) => {
    //         for boat in boats {
    //             println!("{} | {} | {}", boat.id, boat.name, boat.path);
    //         }
    //     },
    //     Err(e) => {
    //         eprintln!("Erreur lors de la récupération : {}", e);
    //     }
    // }


    // GET ONE:
    //
    // match boat.get_boat_by_id(15) {
    //     Ok(boat) => {
    //         println!("{} | {} | {}", boat.id, boat.name, boat.path);
    //     },
    //     Err(e) => {
    //         eprintln!("Erreur lors de la récupération : {}", e);
    //     }
    // }

    // SCRIPT PYTHON:

    // let json_raw = r#"
    // [
    //     {
    //         "timestamp": "11:53:20",
    //         "id": 419366912,
    //         "length": "8",
    //         "message": "b'\\x11\\x01\\x00\\x00\\x00\\x00\\x00\\x00'"
    //     },
    //     {
    //         "timestamp": "11:53:20",
    //         "id": 419366912,
    //         "length": "8",
    //         "message": "b'\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00'"
    //     }
    // ]
    // "#;


    
    // functionDecryptPython("hee".to_string());


    let config = web::Data::new(AppState { boat: Mutex::new(boat), });

    
    

    HttpServer::new(move || {
        App::new()
        .app_data(config.clone())
        .service(
            web::scope("/rust/api")
                .service(index)
                //.service(greet)
                .service(get_boat_one)
                .service(get_grouped_boats)
                .service(get_boat_by_id_post)
                .service(raspberryData)
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}




