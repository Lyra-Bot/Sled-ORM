## Sled ORM for lyra Bot

This is a ORM based in Sled for the proyect Discord Bot [Lyra Bot](https://discord.gg)


## Examples


### Simple Connection
```rust
let temp_dir = tempdir()?;
let db_path = temp_dir.path().join("test_db");

// create the connection
let conn = Connection::new(db_path.to_str().unwrap())?;

// if the connection is success open one tree
let test_tree = conn.db.open_tree(b"test_connection")?;

// Insert an key
test_tree.insert(b"key", b"value")?;

```

### With Models

```rust
struct User {
  id: String
  name:String
}

impl User {
  pub fn new(id: &str, name: &str){
    Self {
      id: id.to_string(),
      name: name.to_string()
    }
  }
}

let temp_dir = tempdir()?;
let db_path = temp_dir.path().join("user_db");

let conn = Connection::new(db_path.to_str().unwrap())?;

let user_tree = conn.db.open_tree(b"users")?;

let user = User::new("1", "Reiner Brawn")

// Insert an User Model
test_tree.insert(&user.id, &user)?;



```


```rust
let retrieved_user = users_tree.get::<_, User>(&user.id)?
    .expect("User Exists");
assert_eq!(retrieved_user, user);
        
```

### Simple Transaction


```rust

tree.transaction(|tx| {
    let current_value: Option<i32> = tx.get(b"balance")?;
    
    if let Some(balance) = current_value {
        if balance < 100 {
            // Rollback manual - saldo insuficiente
            return Err(ConflictableTransactionError::Abort(
                "You dont have money".into()
            ));
        }
        
        // Update Balance
        tx.insert(b"balance", &(balance - 100))?;
    }
    
    Ok(())
})?;

```
