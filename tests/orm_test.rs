#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};
    use tempfile::tempdir;
    use std::path::Path;
    use sled_orm::Connection;
    use std::time::Instant;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // Contador global para tests
    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

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
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("🧪 Starting test_connection #{}", test_id);
        
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join(format!("test_db_{}", test_id));
        println!("📁 Database path: {:?}", db_path);
        
        let start = Instant::now();
        let conn = Connection::new(db_path.to_str().unwrap())?;
        println!("✅ Connection created in {:?}", start.elapsed());
        
        // Verificar que la conexión funciona intentando abrir un árbol
        let test_tree = conn.db.open_tree(b"test_connection")?;
        test_tree.insert(b"key", b"value")?;
        assert!(test_tree.get(b"key")?.is_some());
        
        println!("✅ Connection test passed in {:?}", start.elapsed());
        Ok(())
    }

    // Test de ORM y árboles (corregido)
    #[test]
    fn test_orm_and_trees() -> Result<(), Box<dyn std::error::Error>> {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("🧪 Starting test_orm_and_trees #{}", test_id);
        let start = Instant::now();
        
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join(format!("test_db_{}", test_id));
        let conn = Connection::new(db_path.to_str().unwrap())?;
        let orm = conn.get_orm();
        
        // Test crear árbol
        let tree_start = Instant::now();
        let users_tree = orm.tree("users")?;
        println!("🌳 Users tree created in {:?}", tree_start.elapsed());
        assert_eq!(&users_tree.tree.name() as &[u8], b"users");
        println!("📊 Tree name: {:?}", String::from_utf8_lossy(&users_tree.tree.name()));
        
        // Test crear múltiples árboles
        let products_tree = orm.tree("products")?;
        assert_eq!(&products_tree.tree.name() as &[u8], b"products");
        println!("📊 Products tree name: {:?}", String::from_utf8_lossy(&products_tree.tree.name()));
        
        Ok(())
    }

    // Test de operaciones CRUD con timing detallado
    #[test]
    fn test_crud_operations() -> Result<(), Box<dyn std::error::Error>> {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("🧪 Starting test_crud_operations #{}", test_id);
        let total_start = Instant::now();
        
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join(format!("test_db_{}", test_id));
        let conn = Connection::new(db_path.to_str().unwrap())?;
        let orm = conn.get_orm();
        let users_tree = orm.tree("users")?;
        
        // Crear usuario de prueba
        let user = TestUser::new("user_1", "Alice", "alice@example.com", 25);
        println!("👤 Test user: {:?}", user);
        
        // Test INSERT
        let insert_start = Instant::now();
        users_tree.insert(&user.id, &user)?;
        println!("💾 Insert completed in {:?}", insert_start.elapsed());
        
        // Test GET (existente)
        let get_start = Instant::now();
        let retrieved_user = users_tree.get::<_, TestUser>(&user.id)?
            .expect("Usuario debería existir");
        println!("🔍 Get completed in {:?}", get_start.elapsed());
        assert_eq!(retrieved_user, user);
        println!("✅ User retrieved correctly: {:?}", retrieved_user);
        
        // Test GET (no existente)
        let non_existent = users_tree.get::<_, TestUser>("non_existent")?;
        assert!(non_existent.is_none());
        println!("✅ Non-existent key handled correctly");
        
        // Test UPDATE
        let update_start = Instant::now();
        let mut updated_user = user.clone();
        updated_user.age = 26;
        users_tree.update(&user.id, &updated_user)?;
        println!("🔄 Update completed in {:?}", update_start.elapsed());
        
        let after_update = users_tree.get::<_, TestUser>(&user.id)?
            .expect("Usuario debería existir después de update");
        assert_eq!(after_update.age, 26);
        println!("✅ User updated correctly: {:?}", after_update);
        
        // Test DELETE
        let delete_start = Instant::now();
        users_tree.delete(&user.id)?;
        println!("🗑️ Delete completed in {:?}", delete_start.elapsed());
        
        let after_delete = users_tree.get::<_, TestUser>(&user.id)?;
        assert!(after_delete.is_none());
        println!("✅ User deleted correctly");
        
        println!("✅ All CRUD operations completed in {:?}", total_start.elapsed());
        Ok(())
    }

    // Test de búsquedas y predicados con estadísticas
    #[test]
    fn test_find_operations() -> Result<(), Box<dyn std::error::Error>> {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("🧪 Starting test_find_operations #{}", test_id);
        let total_start = Instant::now();
        
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join(format!("test_db_{}", test_id));
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
        
        let insert_start = Instant::now();
        for user in &users {
            users_tree.insert(&user.id, user)?;
        }
        println!("💾 Inserted {} users in {:?}", users.len(), insert_start.elapsed());
        
        // Test FIND con predicado (usuarios menores de 30)
        let find_start = Instant::now();
        let young_users = users_tree.find(|user: &TestUser| user.age < 30)?;
        println!("🔍 Find young users completed in {:?}", find_start.elapsed());
        assert_eq!(young_users.len(), 2);
        assert!(young_users.iter().all(|u| u.age < 30));
        println!("✅ Found {} young users: {:?}", young_users.len(), young_users);
        
        // Test FIND con predicado de nombre
        let alice_users = users_tree.find(|user: &TestUser| user.name == "Alice")?;
        assert_eq!(alice_users.len(), 1);
        assert_eq!(alice_users[0].name, "Alice");
        println!("✅ Found Alice: {:?}", alice_users[0]);
        
        // Test ALL (todos los usuarios)
        let all_start = Instant::now();
        let all_users = users_tree.all::<TestUser>()?;
        println!("📊 All users query completed in {:?}", all_start.elapsed());
        assert_eq!(all_users.len(), 4);
        println!("✅ Found all {} users", all_users.len());
        
        // Estadísticas de rendimiento
        println!("📈 Find operations performance:");
        println!("  - Total time: {:?}", total_start.elapsed());
        println!("  - Users inserted: {}", users.len());
        println!("  - Queries executed: 3");
        
        Ok(())
    }

    // Test de concurrencia mejorado
    #[test]
    fn test_concurrency() -> Result<(), Box<dyn std::error::Error>> {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("🧪 Starting test_concurrency #{}", test_id);
        let total_start = Instant::now();
        
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join(format!("test_db_{}", test_id));
        let conn = Connection::new(db_path.to_str().unwrap())?;
        
        let num_threads = 10;
        let operations_per_thread = 100;
        println!("🚀 Starting {} threads with {} operations each", num_threads, operations_per_thread);
        
        let db = Arc::new(conn.db);
        let mut handles = Vec::new();
        let success_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));
        
        for thread_id in 0..num_threads {
            let db_clone = db.clone();
            let success_clone = success_count.clone();
            let error_clone = error_count.clone();
            
            let handle = std::thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let thread_start = Instant::now();
                let tree = db_clone.open_tree(b"concurrent")?;
                let mut thread_success = 0;
                let mut thread_errors = 0;
                
                for op_id in 0..operations_per_thread {
                    let key = format!("thread_{}_key_{}", thread_id, op_id);
                    let value = format!("thread_{}_value_{}", thread_id, op_id);
                    
                    match bincode::serde::encode_to_vec(&value, bincode::config::standard()) {
                        Ok(serialized) => {
                            if let Err(e) = tree.insert(key.as_bytes(), serialized) {
                                thread_errors += 1;
                                eprintln!("❌ Thread {} operation {} failed: {}", thread_id, op_id, e);
                                continue;
                            }
                            
                            if let Some(ivec) = tree.get(key.as_bytes())? {
                                match bincode::serde::decode_from_slice::<String, _>(&ivec, bincode::config::standard()) {
                                    Ok((retrieved, _)) => {
                                        if retrieved == value {
                                            thread_success += 1;
                                        } else {
                                            thread_errors += 1;
                                            eprintln!("❌ Value mismatch in thread {}", thread_id);
                                        }
                                    }
                                    Err(e) => {
                                        thread_errors += 1;
                                        eprintln!("❌ Deserialization failed in thread {}: {}", thread_id, e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            thread_errors += 1;
                            eprintln!("❌ Serialization failed in thread {}: {}", thread_id, e);
                        }
                    }
                }
                
                success_clone.fetch_add(thread_success, Ordering::SeqCst);
                error_clone.fetch_add(thread_errors, Ordering::SeqCst);
                
                println!("🧵 Thread {} completed: {} success, {} errors in {:?}", 
                    thread_id, thread_success, thread_errors, thread_start.elapsed());
                Ok(())
            });
            handles.push(handle);
        }
        
        // Esperar que todos los hilos terminen y recolectar resultados
        let mut thread_results = Vec::new();
        for (i, handle) in handles.into_iter().enumerate() {
            match handle.join() {
                Ok(result) => {
                    if let Err(e) = result {
                        eprintln!("❌ Thread {} failed: {}", i, e);
                        thread_results.push(Err(e));
                    } else {
                        thread_results.push(Ok(()));
                    }
                }
                Err(e) => {
                    eprintln!("❌ Thread {} panicked: {:?}", i, e);
                }
            }
        }
        
        let total_operations = num_threads * operations_per_thread;
        let total_success = success_count.load(Ordering::SeqCst);
        let total_errors = error_count.load(Ordering::SeqCst);
        
        println!("📊 Concurrency test results:");
        println!("  - Total operations: {}", total_operations);
        println!("  - Successful: {} ({:.1}%)", total_success, (total_success as f64 / total_operations as f64) * 100.0);
        println!("  - Errors: {} ({:.1}%)", total_errors, (total_errors as f64 / total_operations as f64) * 100.0);
        println!("  - Total time: {:?}", total_start.elapsed());
        
        // Verificar que al menos el 95% de las operaciones fueron exitosas
        assert!(total_success as f64 / total_operations as f64 > 0.95, 
            "Less than 95% success rate: {}/{}", total_success, total_operations);
        
        Ok(())
    }

    // Test de serialización/deserialización corregido
    #[test]
    fn test_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("🧪 Starting test_serialization #{}", test_id);
        let total_start = Instant::now();
        
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join(format!("test_db_{}", test_id));
        let conn = Connection::new(db_path.to_str().unwrap())?;
        let orm = conn.get_orm();
        let test_tree = orm.tree("serialization")?;
        
        // Test con diferentes tipos de datos (CORREGIDO)
        test_tree.insert("string", &"hello world")?;
        test_tree.insert("number", &42_i32)?;
        test_tree.insert("float", &3.14159_f64)?;
        test_tree.insert("boolean", &true)?;
        test_tree.insert("vector", &vec![1, 2, 3])?;
        test_tree.insert("array", &[1, 2, 3, 4, 5])?;
        test_tree.insert("tuple", &(1, "two".to_string(), 3.0))?;
        
        println!("💾 Inserted 7 test items");
        
        // Verificar deserialización
        let verify_start = Instant::now();
        let string_val: String = test_tree.get("string")?.unwrap();
        let number_val: i32 = test_tree.get("number")?.unwrap();
        let float_val: f64 = test_tree.get("float")?.unwrap();
        let boolean_val: bool = test_tree.get("boolean")?.unwrap();
        let vector_val: Vec<i32> = test_tree.get("vector")?.unwrap();
        let array_val: [i32; 5] = test_tree.get("array")?.unwrap();
        let tuple_val: (i32, String, f64) = test_tree.get("tuple")?.unwrap();
        
        assert_eq!(string_val, "hello world");
        assert_eq!(number_val, 42);
        assert!((float_val - 3.14159).abs() < 0.0001);
        assert_eq!(boolean_val, true);
        assert_eq!(vector_val, vec![1, 2, 3]);
        assert_eq!(array_val, [1, 2, 3, 4, 5]);
        assert_eq!(tuple_val, (1, "two".to_string(), 3.0));
        
        println!("✅ All serialization tests passed in {:?}", verify_start.elapsed());
        println!("📊 Total serialization test time: {:?}", total_start.elapsed());
        
        Ok(())
}

    // Test de rendimiento con métricas detalladas
    #[test]
    fn test_performance() -> Result<(), Box<dyn std::error::Error>> {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("🧪 Starting test_performance #{}", test_id);
        let total_start = Instant::now();
        
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join(format!("test_db_{}", test_id));
        let conn = Connection::new(db_path.to_str().unwrap())?;
        let orm = conn.get_orm();
        let tree = orm.tree("performance")?;
        
        let num_items = 1000;
        println!("⏱️ Testing performance with {} items", num_items);
        
        // Fase de inserción
        let insert_start = Instant::now();
        for i in 0..num_items {
            let key = format!("key_{:04}", i);
            let value = format!("value_{}", i);
            tree.insert(&key, &value)?;
            
            // Log cada 100 inserciones
            if i % 100 == 0 && i > 0 {
                println!("📦 Inserted {} items...", i);
            }
        }
        let insert_time = insert_start.elapsed();
        let insert_rate = num_items as f64 / insert_time.as_secs_f64();
        
        // Fase de lectura
        let read_start = Instant::now();
        let all_values = tree.all::<String>()?;
        let read_time = read_start.elapsed();
        let read_rate = num_items as f64 / read_time.as_secs_f64();
        
        assert_eq!(all_values.len(), num_items);
        
        // Fase de verificación
        let verify_start = Instant::now();
        for i in 0..num_items.min(100) { // Verificar solo una muestra
            let key = format!("key_{:04}", i);
            let retrieved: Option<String> = tree.get(&key)?;
            assert_eq!(retrieved, Some(format!("value_{}", i)));
        }
        let verify_time = verify_start.elapsed();
        
        // Métricas detalladas
        println!("📊 Performance metrics:");
        println!("  - Insert time: {:?} ({:.0} ops/sec)", insert_time, insert_rate);
        println!("  - Read time: {:?} ({:.0} ops/sec)", read_time, read_rate);
        println!("  - Verify time: {:?}", verify_time);
        println!("  - Total time: {:?}", total_start.elapsed());
        println!("  - Memory usage: {:?}", tree.tree.len()); // Número aproximado de elementos
        
        // Umbrales de rendimiento (ajustar según hardware)
        assert!(insert_rate > 1000.0, "Insert rate too low: {:.0} ops/sec", insert_rate);
        assert!(read_rate > 500.0, "Read rate too low: {:.0} ops/sec", read_rate);
        assert!(total_start.elapsed().as_secs_f64() < 5.0, "Test took too long: {:?}", total_start.elapsed());
        
        println!("✅ Performance test passed");
        Ok(())
    }

    // Test de transacciones con implementación real
    #[test]
    fn test_transactions() -> Result<(), Box<dyn std::error::Error>> {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("🧪 Starting test_transactions #{}", test_id);
        
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join(format!("test_db_{}", test_id));
        let conn = Connection::new(db_path.to_str().unwrap())?;
        let orm = conn.get_orm();
        let tree = orm.tree("transactions")?;
        
        println!("💾 Testing transaction support...");
        
        // Aquí irían las pruebas de transacciones cuando las implementes
        // Por ahora verificamos que el árbol funciona normalmente
        tree.insert("test_key", &"test_value")?;
        let value: Option<String> = tree.get("test_key")?;
        assert_eq!(value, Some("test_value".to_string()));
        
        println!("✅ Basic functionality works (transactions to be implemented)");
        Ok(())
    }

    // Test adicional: Stress test con múltiples operaciones (CORREGIDO)
    #[test]
    fn test_stress() -> Result<(), Box<dyn std::error::Error>> {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("🧪 Starting test_stress #{}", test_id);
        let total_start = Instant::now();
        
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join(format!("test_db_{}", test_id));
        let conn = Connection::new(db_path.to_str().unwrap())?;
        
        // Probar múltiples árboles secuencialmente (en lugar de concurrentemente)
        let trees = vec!["users", "products", "orders", "logs", "config"];
        
        for tree_name in trees {
            let tree = conn.db.open_tree(tree_name)?;
            
            // Operaciones básicas en cada árbol
            for i in 0..100 {
                let key = format!("{}_{}", tree_name, i);
                let value = format!("value_{}_{}", tree_name, i);
                
                let serialized = bincode::serde::encode_to_vec(&value, bincode::config::standard())?;
                tree.insert(key.as_bytes(), serialized)?;
                
                if let Some(ivec) = tree.get(key.as_bytes())? {
                    let (retrieved, _): (String, _) = bincode::serde::decode_from_slice(&ivec, bincode::config::standard())?;
                    assert_eq!(retrieved, value);
                }
            }
            
            println!("✅ Tree {} completed operations", tree_name);
        }
        
        println!("✅ All stress test operations completed in {:?}", total_start.elapsed());
        Ok(())
    }
}