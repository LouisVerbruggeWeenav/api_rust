


use actix_web::{get, post, web, App, HttpServer, Responder, Result, web::Json};
use serde_json::{json, Value};

use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::fs;

use std::ffi::CString;



// Ensure the database module is declared

mod database;
use crate::database::{boat::Boat, connection::Connection};
use inline_python::python;

use serde::Deserialize;

#[derive(Deserialize)]
struct Info {
    id: i32,
}



#[post("/api/boats/one")]
async fn get_boat_one(info: web::Json<Info>) -> Result<String> {
    
    Ok(format!("Welcome {}!", info.id))

}



#[get("/")]
async fn index() -> impl Responder {

    let nom_fichier = "../boats/test/test.json";

    let contenu = fs::read_to_string(nom_fichier).expect("Quelque chose s'est mal passé lors de la lecture du fichier");
    let json: serde_json::Value = serde_json::from_str(&contenu).expect("msg");

    Json(
        json
    )
}



#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {


    format!("Hello {name}!")
}




fn function_python() -> PyResult<()> {
    let code_str = fs::read_to_string("./main.py").expect("Fichier Python introuvable");

    let code_cstring = CString::new(code_str).expect("CString::new failed");
    let filename = CString::new("main.py").unwrap();
    let modulename = CString::new("main").unwrap();

    Python::with_gil(|py| {
        let module = PyModule::from_code(py, code_cstring.as_c_str(), filename.as_c_str(), modulename.as_c_str())?;
        module.getattr("run")?.call1(("hello louis".to_string(), ))?;
        Ok(())
    })
}



#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
// fn main() {
    let mut database = Connection::new("localhost".to_string(), 3306, "root".to_string(), "welcome1".to_string(), "boat_directory".to_string());
    database.connect();

    // get all data boats:
    let mut boat = Boat::new(database.getConn());





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
    //
    // function_python();



    HttpServer::new(|| {
        App::new()
        .service(
            web::scope("/rust")
                .service(index)
                .service(greet)
                .service(get_boat_one)
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}





