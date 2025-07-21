

use mysql::{Opts, Pool, PooledConn};
use urlencoding::encode;

pub struct Connection {
    pool: Pool,
}

impl Connection { 
    pub fn new(host: String, port: String, user: String, password: String, database: String) -> Result<Self, Box<dyn std::error::Error>> {

        let url = format!("mysql://{}:{}@{}:{}/{}", encode(&user), encode(&password), host, port, database);
        let opts: Opts = Opts::from_url(&url)?;
        let pool: Pool = Pool::new(opts)?;
        Ok(Connection { pool })

    }

    pub fn get_pool(&self) -> &Pool {
        &self.pool
    }

    // pub fn connect(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {

    //     let opts: Opts = Opts::from_url(self.url.as_str()).expect("Failed to parse URL");
    //     self.pool = Pool = Pool::new(opts).expect("Failed to create pool");
    //     // let conn: PooledConn = pool.get_conn().expect("Failed to get connection");
    //     // self.conn = Some(conn);
        
    //     println!("Connected to the database successfully!");

    //     Ok(())

    // }

    pub fn get_conn(&self) -> Result<PooledConn, Box<dyn std::error::Error>> {
        Ok(self.pool.get_conn()?)
    }

}