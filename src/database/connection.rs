

use mysql::{Opts, Pool, PooledConn};


pub struct Connection {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub conn: Option<PooledConn>,
}


impl Connection { 
    pub fn new(host: String, port: String, user: String, password: String, database: String) -> Self {
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

        Ok(())

    }

    pub fn getConn(&mut self) -> Option<PooledConn> {
        self.conn.take()
    }

}