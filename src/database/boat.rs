

use mysql::{Pool, PooledConn};
use mysql::prelude::*;
use mysql::params;
use serde::{Serialize};
use std::io::Write;
use std::fs::{self, File};
use std::path::Path;



use crate::database::boat;


#[derive(Debug)]
pub struct Boat {
    conn: Pool,
}

#[derive(Debug, Serialize, FromRow)]
pub struct BoatCollection  {
    pub id: u32,
    pub name: String,
    pub path: String
}

    #[derive(Debug, Serialize)]
    pub struct BoatCount {
        pub name: String,
        pub count: u32
    }


impl Boat {
    pub fn new(conn_result: Result<Pool, Box<dyn std::error::Error>>) -> Self {

        let conn = conn_result.expect("Impossible de récupérer une connexion");

        Boat { conn, }
    }

    pub fn add_boat(&mut self, name: String, startRecord: String, endRecord: String, dataStruct: serde_json::Value) -> Result<bool, Box<dyn std::error::Error>> {

        // add file
        let path_str = format!("./boats/{}", name);
        let path = Path::new(&path_str);

        if !path.exists() {
            fs::create_dir_all(path)?;
            println!("Dossier créé : {}", path.display());
        } else {
            println!("Le dossier existe déjà : {}", path.display());
        }

        let file_path = path.join(format!("{}.json", startRecord ));
        let json_string = serde_json::to_string_pretty(&dataStruct)
            .expect("Erreur lors de la conversion en JSON");
        let mut file = File::create(&file_path)?;
        file.write_all(json_string.as_bytes())?;

        println!("Données enregistrées dans {}", file_path.display());

        // add to database:
        let mut conn = self.conn.get_conn()?;

        let result = conn.exec_drop(
            "INSERT INTO boats (name, path, endRecord) VALUES (?, ?, ?)",
            (name, startRecord, endRecord),
        );

        if let Err(e) = result {
            println!("Erreur lors de l'insertion en base : {e}");
            return Ok(false);
        }

        Ok(true)
    }

    pub fn get_boat_by_id(&mut self, id: i32) -> Result<BoatCollection, Box<dyn std::error::Error>> {

        let mut conn = self.conn.get_conn()?;

        let boat = conn
            .exec_first("SELECT id, name, path FROM boats WHERE id = ?;", 
                (id, ),
            )?
            .ok_or("Boat not found")?;

        let (id, name, path): (u32, String, String) = boat;
        Ok(BoatCollection { id, name, path })
    }

    pub fn get_boat_by_different_id(&mut self, listId: Vec<i32>) -> Result<Vec<String>, Box<dyn std::error::Error>> {

        let mut in_list = String::from("(");
        for (i, id) in listId.iter().enumerate() {
            if i > 0 {
                in_list.push_str(", ");
            }
            in_list.push_str(&id.to_string());
        }
        in_list.push(')');

        let query = format!(
            "SELECT id, name, path FROM boats WHERE id IN {};",
            in_list
        );

        let mut conn = self.conn.get_conn()?;

        // Exécuter la requête:
        let rows: Vec<BoatCollection> = conn.query(query)?;

        let mut response: Vec<String> = Vec::new();

        for bo in &rows {
            response.push(format!("boats/{}/{}.json", bo.name, bo.path));
        }


        Ok(response)

    }

    pub fn get_grouped_boats(&mut self) -> Result<Vec<BoatCount>, Box<dyn std::error::Error + Send + Sync>> {

        let mut conn = self.conn.get_conn()?;

        let groupBoats: Vec<BoatCount> = conn
            .query_map(
                "SELECT name, COUNT(name) FROM boats GROUP BY name",
                |(name, count) | BoatCount {name, count}
            )?;
        Ok(groupBoats)
    }

    pub fn get_boat_by_name(&mut self, nameBoat: String) -> Result<Vec<BoatCollection>, Box<dyn std::error::Error>> {

        let mut conn = self.conn.get_conn()?;

        let groupBoats: Vec<BoatCollection> = conn
            .exec_map(
                "SELECT id, name, path FROM boats WHERE name =?;",
                (nameBoat, ),
                |(id, name, path) | BoatCollection {id, name, path}
            )?;
        Ok(groupBoats)
    }
    
}