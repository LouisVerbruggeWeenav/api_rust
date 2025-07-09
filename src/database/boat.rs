
use std::io;

use mysql::{Pool, PooledConn};
use mysql::prelude::*;
use mysql::params;



#[derive(Debug)]
pub struct Boat<'a> {
    conn: Option<&'a mut PooledConn>,
}



#[derive(Debug)]
pub struct BoatCollection  {
    pub id: u32,
    pub name: String,
    pub path: String
}


impl<'a> Boat<'a> {
    pub fn new(conn: Option<&'a mut PooledConn>) -> Self {
        Boat { conn }
    }
    
    pub fn get_all_boats(&mut self) -> Result<Vec<BoatCollection>, Box<dyn std::error::Error>> {

        let conn = self.conn.as_mut(); // 2e Option<&mut>
        let conn = conn.ok_or("conn is None")?; // on extrait le &mut PooledConn final

    
        let boats: Vec<BoatCollection> = conn
            .query_map(
                "SELECT id, name, path FROM boats",
                |(id, name, path) | BoatCollection {id, name, path}
            )?;

        Ok(boats)
    }

    pub fn get_boat_by_id(&mut self, id: i32) -> Result<BoatCollection, Box<dyn std::error::Error>> {

        let conn = self.conn.as_mut();
        let conn = conn.ok_or("conn is None")?;

        let boat = conn
            .exec_first("SELECT id, name, path FROM boats WHERE id=:id;", 
                params! (
                    "id" => id
                ),
            )?
            .ok_or("Boat not found")?;

        let (id, name, path): (u32, String, String) = boat;


        Ok(BoatCollection { id, name, path })
    }

}