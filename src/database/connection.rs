

use mysql::{Opts, Pool, PooledConn};
use urlencoding::encode;

pub struct Connection {
    host: String,
    port: String,
    user: String,
    password: String,
    database: String,
    conn: Option<PooledConn>,
    url: String
}


impl Connection { 
    pub fn new(host: String, port: String, user: String, password: String, database: String) -> Self {

        let url = format!("mysql://{}:{}@{}:{}/{}", encode(&user), encode(&password), host, port, database);
        Connection { 
            host,
            port,
            user,
            password,
            database,
            conn: None,
            url
            
        }
    }

    pub fn connect(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {

        let opts: Opts = Opts::from_url(self.url.as_str()).expect("Failed to parse URL");
        let pool: Pool = Pool::new(opts).expect("Failed to create pool");
        let conn: PooledConn = pool.get_conn().expect("Failed to get connection");
        self.conn = Some(conn);
        
        println!("Connected to the database successfully!");

        Ok(())

    }

    pub fn getConn(&mut self) -> Option<PooledConn> {
        self.conn.take()
    }

}