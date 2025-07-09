

use mysql::{Opts, Pool, PooledConn};

use mysql::*;
use mysql::prelude::*;


pub struct Connection {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub conn: Option<PooledConn>,
}



struct ListBoat {
    id: u32,
    name: String,
    path: String
}

impl Connection { 
    pub fn new(host: String, port: u16, user: String, password: String, database: String) -> Self {
        Connection { 
            host: host,
            port: port,
            user: user,
            password: password,
            database: database,
            conn: None,
            
        }
    }

    pub fn connect(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {

        // let url: String = format!("mysql://{}:{}@{}:{}/{}", self.user, self.password, self.host, self.port, self.database);
        let url: &'static str = "mysql://root:welcome1@localhost:3306/boat_directory";
        let opts: Opts = Opts::from_url(url).expect("Failed to parse URL");
        let pool: Pool = Pool::new(opts).expect("Failed to create pool");
        let conn: PooledConn = pool.get_conn().expect("Failed to get connection");
        self.conn = Some(conn);
        
        println!("Connected to the database successfully!");

        // conn.exec_drop(
        //     r"INSERT INTO boats (name, path) VALUES (:name, :path)",
        //     params! {
        //         "name" => "name 4",
        //         "path" => "path 4",
        //     }
        // )?;

        Ok(())

    }

    pub fn getConn(&mut self) -> Option<&mut PooledConn> {
        return self.conn.as_mut();
    }
}