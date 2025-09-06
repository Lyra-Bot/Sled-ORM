use crate::{Tree, ORM,Connection};


impl ORM {
    pub fn tree(&self, name: &str) -> Result<Tree, sled::Error> {
        let tree = self.conn.db.open_tree(name.as_bytes())?;
        Ok(Tree { 
            conn: Connection { db: self.conn.db.clone() }, 
            tree 
        })
    }
}