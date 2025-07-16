


use actix_web::{get, post, web, App, HttpServer, Responder, web::Json};
use serde_json;

use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::{f32::consts::E, fs};

use std::ffi::CString;

use std::sync::Mutex;
use serde_json::Value;
use serde::{Serialize, Deserialize};


// Ensure the database module is declared

mod database;
use crate::database::{boat::Boat, connection::Connection};
use std::cell::RefCell;


use dotenv::dotenv;
use std::env;



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
    startRecord: String,
    endRecord: String
}


#[derive(Deserialize, Clone)]
struct InfoRaspberrypi{
    infoBoat: StructInfoBoat,
    structData: String
}

#[derive(Deserialize, Clone)]
struct InfoFrontByName {
    name: String,
}


#[post("/raspberrypi/data")]
async fn raspberryData(data: web::Data<AppState>, info: web::Json<InfoRaspberrypi>) ->  impl Responder {

    let data_struct: Value = functionDecryptPython(info.structData.clone()).expect("msg");
    let mut boat = data.boat.lock().unwrap();
    boat.add_boat(info.infoBoat.name.clone(), info.infoBoat.startRecord.clone(), info.infoBoat.endRecord.clone(), data_struct);
    "Succes"
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

    let code_str = fs::read_to_string("./src/decryp/decryp.py")
        .expect("Fichier Python introuvable");

    let code_cstring = CString::new(code_str).expect("CString::new failed");
    let filename = CString::new("decryp.py").unwrap();
    let modulename = CString::new("main").unwrap();


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

            *parsed.borrow_mut() = value;
            Ok(())
        })();

        if let Err(e) = result {
            e.print(py);
        }
    });

    Ok(parsed.into_inner())
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
// fn main() {

    dotenv().ok();

    let host = env::var("DB_HOST").expect("DB_HOST must be set");
    let port = env::var("DB_PORT").expect("DB_PORT must be set");
    let user = env::var("DB_USER").expect("DB_USER must be set");
    let password = env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
    let database = env::var("DB_DATABASE").expect("DB_DATABASE must be set");


    // env::var("DB_USER")

    let mut database = Connection::new(host, port, user, password, database);
    database.connect();

    let mut boat = Boat::new(database.getConn());
    let config = web::Data::new(AppState { boat: Mutex::new(boat), });

    HttpServer::new(move || {
        App::new()
        .app_data(config.clone())
        .app_data(web::PayloadConfig::new(1024 * 1024 * 1024)) // = 1Go
        .service(
            web::scope("/api")
                .service(get_boat_one)
                .service(get_grouped_boats)
                .service(get_boat_by_id_post)
                .service(raspberryData)
        )
    })
    //.bind(("127.0.0.1", 8080))?
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[get("/boats/grouped")]



