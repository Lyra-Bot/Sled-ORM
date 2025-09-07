#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};
    use tempfile::tempdir;
    use std::path::Path;
    use sled_orm::Connection;
    // Modelo de prueba
    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestUser {
        id: String,
        name: String,
        email: String,
        age: u32,
    }

    impl TestUser {
        fn new(id: &str, name: &str, email: &str, age: u32) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                email: email.to_string(),
                age,
            }
        }
    }

    // Test de conexión
    #[test]
    fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");
        
        // Test crear conexión
        let conn = Connection::new(db_path.to_str().unwrap())?;
        
        // Verificar que la conexión funciona intentando abrir un árbol
        let test_tree = conn.db.open_tree(b"test_connection")?;
        test_tree.insert(b"key", b"value")?;
        assert!(test_tree.get(b"key")?.is_some());
        
        Ok(())
    }

    // Test de ORM y árboles
    #[test]
    fn test_orm_and_trees() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");
        let conn = Connection::new(db_path.to_str().unwrap())?;
        let orm = conn.get_orm();
        
        // Test crear árbol
        let users_tree = orm.tree("users")?;
        assert_eq!(users_tree.tree.name(), b"users");
        
        // Test crear múltiples árboles
        let products_tree = orm.tree("products")?;
        assert_eq!(products_tree.tree.name(), b"products");
        
        Ok(())
    }

    // Test de operaciones CRUD
    #[test]
    fn test_crud_operations() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");
        let conn = Connection::new(db_path.to_str().unwrap())?;
        let orm = conn.get_orm();
        let users_tree = orm.tree("users")?;
        
        // Crear usuario de prueba
        let user = TestUser::new("user_1", "Alice", "alice@example.com", 25);
        
        // Test INSERT
        users_tree.insert(&user.id, &user)?;
        
        // Test GET (existente)
        let retrieved_user = users_tree.get::<_, TestUser>(&user.id)?
            .expect("Usuario debería existir");
        assert_eq!(retrieved_user, user);
        
        // Test GET (no existente)
        let non_existent = users_tree.get::<_, TestUser>("non_existent")?;
        assert!(non_existent.is_none());
        
        // Test UPDATE
        let mut updated_user = user.clone();
        updated_user.age = 26;
        users_tree.update(&user.id, &updated_user)?;
        
        let after_update = users_tree.get::<_, TestUser>(&user.id)?
            .expect("Usuario debería existir después de update");
        assert_eq!(after_update.age, 26);
        
        // Test DELETE
        users_tree.delete(&user.id)?;
        let after_delete = users_tree.get::<_, TestUser>(&user.id)?;
        assert!(after_delete.is_none());
        
        Ok(())
    }

    // Test de búsquedas y predicados
    #[test]
    fn test_find_operations() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");
        let conn = Connection::new(db_path.to_str().unwrap())?;
        let orm = conn.get_orm();
        let users_tree = orm.tree("users")?;
        
        // Insertar varios usuarios
        let users = vec![
            TestUser::new("user_1", "Alice", "alice@example.com", 25),
            TestUser::new("user_2", "Bob", "bob@example.com", 30),
            TestUser::new("user_3", "Charlie", "charlie@example.com", 22),
            TestUser::new("user_4", "Diana", "diana@example.com", 35),
        ];
        
        for user in &users {
            users_tree.insert(&user.id, user)?;
        }
        
        // Test FIND con predicado (usuarios menores de 30)
        let young_users = users_tree.find(|user: &TestUser| user.age < 30)?;
        assert_eq!(young_users.len(), 2);
        assert!(young_users.iter().all(|u| u.age < 30));
        
        // Test FIND con predicado de nombre
        let alice_users = users_tree.find(|user: &TestUser| user.name == "Alice")?;
        assert_eq!(alice_users.len(), 1);
        assert_eq!(alice_users[0].name, "Alice");
        
        // Test ALL (todos los usuarios)
        let all_users = users_tree.all::<TestUser>()?;
        assert_eq!(all_users.len(), 4);
        
        Ok(())
    }

    // Test de concurrencia
    #[test]
    fn test_concurrency() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");
        let conn = Connection::new(db_path.to_str().unwrap())?;
        
        // Usar Arc para compartir la conexión de manera segura
        let db = std::sync::Arc::new(conn.db);
        let mut handles = Vec::new();
        
        for i in 0..10 {
            let db_clone = db.clone();
            let handle = std::thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let tree = db_clone.open_tree(b"concurrent")?;
                let key = format!("key_{}", i);
                let value = format!("value_{}", i);
                
                // Serializar el valor
                let serialized = bincode::serde::encode_to_vec(&value, bincode::config::standard())?;
                tree.insert(key.as_bytes(), serialized)?;
                
                // Verificar que se insertó correctamente
                if let Some(ivec) = tree.get(key.as_bytes())? {
                    let (retrieved, _): (String, _) = bincode::serde::decode_from_slice(&ivec, bincode::config::standard())?;
                    assert_eq!(retrieved, value);
                }
                
                Ok(())
            });
            handles.push(handle);
        }
        
        // Esperar que todos los hilos terminen
        for handle in handles {
            handle.join().unwrap();
        }
        
        Ok(())
    }

    // Test de serialización/deserialización
    #[test]
    fn test_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");
        let conn = Connection::new(db_path.to_str().unwrap())?;
        let orm = conn.get_orm();
        let test_tree = orm.tree("serialization")?;
        
        // Test con diferentes tipos de datos
        test_tree.insert("string", &"hello world")?;
        test_tree.insert("number", &42)?;
        test_tree.insert("boolean", &true)?;
        test_tree.insert("vector", &vec![1, 2, 3])?;
        
        // Verificar deserialización
        let string_val: String = test_tree.get("string")?.unwrap();
        let number_val: i32 = test_tree.get("number")?.unwrap();
        let boolean_val: bool = test_tree.get("boolean")?.unwrap();
        let vector_val: Vec<i32> = test_tree.get("vector")?.unwrap();
        
        assert_eq!(string_val, "hello world");
        assert_eq!(number_val, 42);
        assert_eq!(boolean_val, true);
        assert_eq!(vector_val, vec![1, 2, 3]);
        
        Ok(())
    }

    // Test de manejo de errores
    #[test]
    fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");
        let conn = Connection::new(db_path.to_str().unwrap())?;
        let orm = conn.get_orm();
        let tree = orm.tree("errors")?;
        
        // Test que insertar con clave vacía funciona
        tree.insert("", &"empty key")?;
        let empty_val: String = tree.get("")?.unwrap();
        assert_eq!(empty_val, "empty key");
        
        // Test que no se puede deserializar datos corruptos
        tree.tree.insert("corrupt", b"invalid bincode data")?;
        let result: Result<Option<String>, _> = tree.get("corrupt");
        assert!(result.is_err()); // Debería fallar la deserialización
        
        Ok(())
    }

    // Test de rendimiento con muchos datos
    #[test]
    fn test_performance() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");
        let conn = Connection::new(db_path.to_str().unwrap())?;
        let orm = conn.get_orm();
        let tree = orm.tree("performance")?;
        
        let start = std::time::Instant::now();
        
        // Insertar 1000 items
        for i in 0..1000 {
            let key = format!("key_{:04}", i);
            let value = format!("value_{}", i);
            tree.insert(&key, &value)?;
        }
        
        let insert_time = start.elapsed();
        println!("Tiempo de inserción de 1000 items: {:?}", insert_time);
        
        // Leer todos los items
        let all_values = tree.all::<String>()?;
        assert_eq!(all_values.len(), 1000);
        
        let total_time = start.elapsed();
        println!("Tiempo total: {:?}", total_time);
        
        Ok(())
    }

    // Test de transacciones (si las implementas)
    #[test]
    fn test_transactions() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");
        let conn = Connection::new(db_path.to_str().unwrap())?;
        let orm = conn.get_orm();
        let tree = orm.tree("transactions")?;
        
        // Este test asume que implementarás transacciones después
        // Por ahora es un placeholder
        println!("Las transacciones se implementarán en una versión futura");
        
        Ok(())
    }
}