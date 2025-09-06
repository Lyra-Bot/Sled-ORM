

use sled::Db;

use crate::{Connection, ORM};

impl Connection {
    pub fn new(path: &str) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        Ok(Connection { db })
    }

    pub fn get_instance(&self) -> &Db {
        &self.db
    }

    pub fn get_orm(&self) -> ORM {
        ORM { conn: Connection { db: self.db.clone() } }
    }
}