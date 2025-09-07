use bincode::config::{BigEndian, Configuration, Fixint};

mod connection;
mod orm;
mod trees;
mod macros;


pub struct Connection {
    pub db: sled::Db
}

pub struct ORM {
    pub conn: Connection
}




pub struct Tree {
    pub conn: Connection,
    pub tree: sled::Tree
}

pub fn bincode_get_config() -> Configuration<BigEndian, Fixint>{
    bincode::config::standard()
        .with_big_endian()
        .with_fixed_int_encoding()
        
}