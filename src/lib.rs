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

pub struct TransactionalTreeWrapper<'a> {
    tree: &'a sled::transaction::TransactionalTree,
}

pub struct Tree {
    pub conn: Connection,
    pub tree: sled::Tree
}