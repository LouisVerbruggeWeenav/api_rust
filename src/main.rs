


use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder, web::Json};
use serde_json;

use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::{f32::consts::E, fs};
use actix_web::middleware::Compress;

use std::ffi::CString;

use std::sync::Mutex;
use serde_json::Value;
use serde::{Serialize, Deserialize};

use actix_web::http::header::{CONTENT_ENCODING, CONTENT_TYPE};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;


// Ensure the database module is declared

mod database;
use crate::database::{boat::Boat, connection::Connection};
use std::cell::RefCell;


use dotenv::dotenv;
use std::env;

use crate::database::boat::BoatCount;


struct AppState {
    boat: Mutex<Boat>,
}


#[derive(Deserialize)]
struct InfoFrontOne {
    id: i32,
}

#[derive(Deserialize, Clone)]
struct InfoFrontConcatOne {
    listPath: Vec<i32>
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

    let data_struct: Value = functionDecryptPython(info.structData.clone()).expect("Erreur l'hors de l'execution du script python 'decryp'");
    let mut boat = data.boat.lock().unwrap();
    boat.add_boat(info.infoBoat.name.clone(), info.infoBoat.startRecord.clone(), info.infoBoat.endRecord.clone(), data_struct);
    "Succes"
}

#[get("/boats/grouped")]
async fn get_grouped_boats(data: web::Data<AppState>) -> impl Responder {
    let data_cloned = data.clone();

    let result = web::block(move || {
        match data_cloned.boat.try_lock() {
            Ok(mut boat) => boat.get_grouped_boats(),
            Err(_) => Err(Box::<dyn std::error::Error + Send + Sync>::from("resource locked")),
        }
    })
    .await;

    let json = match result {
        Ok(Ok(grouped_boats)) => match serde_json::to_value(&grouped_boats) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Erreur de sérialisation: {}", e);
                serde_json::json!({ "error": format!("Erreur de sérialisation: {}", e) })
            }
        },
        Ok(Err(e)) => {
            eprintln!("Erreur dans get_grouped_boats: {}", e);
            serde_json::json!({ "error": format!("Erreur interne: {}", e) })
        }
        Err(e) => {
            eprintln!("Erreur web::block: {:?}", e);
            serde_json::json!({ "error": "Erreur interne serveur" })
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

// #[get("/test")]
// async fn test() -> impl Responder {

//     let testliste = ["boats/Boat_1/2025-07-15-12_28.json", "boats/Boat_1/2025-07-15-12_29.json", "boats/Boat_1/2025-07-15-12_30.json"];
//     let data_concat: serde_json::Value = functionConcatPython(testliste).expect("Erreur l'hors de l'execution du script python 'concat'");


//     Json(serde_json::json!(data_concat))

// }


#[post("/boats/concatOne")]
async fn concatOne(data: web::Data<AppState>, info: web::Json<InfoFrontConcatOne>) -> impl Responder {

    let mut boat = data.boat.lock().unwrap();

    let mut boatsId = boat.get_boat_by_different_id(info.listPath.clone()).expect("mm"); //.lock().unwrap();;

    let data_concat: serde_json::Value = functionConcatPython(boatsId).expect("Erreur l'hors de l'execution du script python 'concat'");
    Json(serde_json::json!(data_concat))


}


#[post("/boats/one")]
async fn get_boat_one(data: web::Data<AppState>, info: web::Json<InfoFrontOne>) -> impl Responder {
    let mut boat = data.boat.lock().unwrap();

    match boat.get_boat_by_id(info.id) {
        Ok(boats) => {
            let path = format!("./boats/{}/{}.json", boats.name, boats.path);
            match tokio::fs::read(&path).await {
                Ok(bytes) => {
                    // Compression gzip
                    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                    if let Err(e) = encoder.write_all(&bytes) {
                        eprintln!("Erreur compression gzip: {}", e);
                        return HttpResponse::InternalServerError().finish();
                    }
                    let compressed_bytes = match encoder.finish() {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("Erreur fin compression gzip: {}", e);
                            return HttpResponse::InternalServerError().finish();
                        }
                    };

                    HttpResponse::Ok()
                        .insert_header((CONTENT_TYPE, "application/json"))
                        .insert_header((CONTENT_ENCODING, "gzip"))
                        .body(compressed_bytes)
                }
                Err(e) => {
                    eprintln!("Erreur lecture fichier {} : {}", path, e);
                    HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": format!("File read failed: {}", e) }))
                }
            }
        }
        Err(e) => {
            eprintln!("Erreur récupération boat : {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": format!("{}", e) }))
        }
    }
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

fn functionConcatPython(listPath: Vec<String>) -> Result<serde_json::Value, Box<dyn std::error::Error>> { 

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

            let result = module.getattr("concatJson")?.call1((listPath, ))?;
            let json_str: String = result.extract().expect("Erreur convert to string");
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


    let database = Connection::new(host, port, user, password, database).expect("Impossible de créer la connexion");
    let pool = database.get_pool().clone();

    let mut boat = Boat::new(Ok(pool));
    let config = web::Data::new(AppState { boat: Mutex::new(boat), });


    HttpServer::new(move || {
        App::new()
        .app_data(config.clone())
        .app_data(web::PayloadConfig::new(1024 * 1024 * 1024)) // = 1Go
        .service(
            web::scope("/api")  

                .wrap(Compress::default())
                .service(get_boat_one)
                .service(get_grouped_boats)
                .service(get_boat_by_id_post)
                .service(raspberryData)

                .service(concatOne)
        )
    })
    //.bind(("127.0.0.1", 8080))?
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}


