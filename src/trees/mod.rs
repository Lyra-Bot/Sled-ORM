use crate::Tree;
use serde::{Serialize, Deserialize};
use bincode;
use sled::IVec;

impl Tree {
    pub fn insert<K, V>(&self, key: K, value: &V) -> Result<(), Box<dyn std::error::Error>>
    where
        K: AsRef<[u8]>,
        V: Serialize,
    {
        // Usar la función correcta de bincode::serde
        let serialized = bincode::serde::encode_to_vec(value, bincode::config::standard())?;
        self.tree.insert(key, IVec::from(serialized))?;
        Ok(())
    }

    pub fn get<K, V>(&self, key: K) -> Result<Option<V>, Box<dyn std::error::Error>>
    where
        K: AsRef<[u8]>,
        V: for<'de> Deserialize<'de>,
    {
        if let Some(ivec) = self.tree.get(key)? {
            // decode_from_slice devuelve (T, usize), extraer solo el valor
            let (value, _) = bincode::serde::decode_from_slice(&ivec, bincode::config::standard())?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn find<F, V>(&self, predicate: F) -> Result<Vec<V>, Box<dyn std::error::Error>>
    where
        V: for<'de> Deserialize<'de>,
        F: Fn(&V) -> bool,
    {
        let mut results = Vec::new();
        
        for item in self.tree.iter() {
            let (_, value) = item?;
            let (deserialized, _) = bincode::serde::decode_from_slice(&value, bincode::config::standard())?;
            
            if predicate(&deserialized) {
                results.push(deserialized);
            }
        }
        
        Ok(results)
    }

    pub fn update<K, V>(&self, key: K, value: &V) -> Result<(), Box<dyn std::error::Error>>
    where
        K: AsRef<[u8]>,
        V: Serialize,
    {
        // Para update, simplemente insertamos de nuevo (sobrescribe)
        self.insert(key, value)
    }

    pub fn delete<K: AsRef<[u8]>>(&self, key: K) -> Result<(), Box<dyn std::error::Error>> {
        self.tree.remove(key)?;
        Ok(())
    }

    pub fn all<V>(&self) -> Result<Vec<V>, Box<dyn std::error::Error>>
    where
        V: for<'de> Deserialize<'de>,
    {
        let mut results = Vec::new();
        
        for item in self.tree.iter() {
            let (_, value) = item?;
            let (deserialized, _) = bincode::serde::decode_from_slice(&value, bincode::config::standard())?;
            results.push(deserialized);
        }
        
        Ok(results)
    }

     pub fn transaction<F, T, E>(
        &self,
        f: F
    ) -> TransactionResult<T, E>
    where
        F: Fn(&TransactionalTree) -> ConflictableTransactionResult<T, E>,
        E: From<Box<dyn std::error::Error>>,
    {
        self.tree.transaction(|tx_tree| {
            // Crear un TransactionalTree wrapper para nuestra API
            let wrapper = TransactionalTreeWrapper { tree: tx_tree };
            f(&wrapper)
        })
    }

    /// Ejecuta una transacción multi-árbol
    pub fn multi_tree_transaction<F, T, E>(
        trees: &[&Tree],
        f: F
    ) -> TransactionResult<T, E>
    where
        F: Fn(&[TransactionalTree]) -> ConflictableTransactionResult<T, E>,
        E: From<Box<dyn std::error::Error>>,
    {
        // Convertir array de Tree a tupla para sled
        let tree_refs: Vec<&sled::Tree> = trees.iter().map(|t| &t.tree).collect();
        
        // Usar la transacción multi-árbol de sled
        match trees.len() {
            1 => tree_refs[0].transaction(|tx| {
                let wrappers = [TransactionalTreeWrapper { tree: tx }];
                f(&wrappers)
            }),
            2 => (tree_refs[0], tree_refs[1]).transaction(|(tx1, tx2)| {
                let wrappers = [
                    TransactionalTreeWrapper { tree: tx1 },
                    TransactionalTreeWrapper { tree: tx2 }
                ];
                f(&wrappers)
            }),
            // Soporte para más árboles si es necesario
            _ => {
                // Para múltiples árboles, necesitamos una implementación más compleja
                Err(TransactionError::Abort(
                    "Multi-tree transactions limited to 2 trees".into()
                ))
            }
        }
    }
}

impl<'a> TransactionalTreeWrapper<'a> {
    /// Insertar un valor serializado en la transacción
    pub fn insert<K, V>(&self, key: K, value: &V) -> ConflictableTransactionResult<(), Box<dyn std::error::Error>>
    where
        K: AsRef<[u8]>,
        V: Serialize,
    {
        let serialized = bincode::serde::encode_to_vec(value, bincode::config::standard())?;
        self.tree.insert(key, serialized)?;
        Ok(())
    }

    /// Obtener un valor deserializado desde la transacción
    pub fn get<K, V>(&self, key: K) -> ConflictableTransactionResult<Option<V>, Box<dyn std::error::Error>>
    where
        K: AsRef<[u8]>,
        V: for<'de> Deserialize<'de>,
    {
        if let Some(ivec) = self.tree.get(key)? {
            let (value, _) = bincode::serde::decode_from_slice(&ivec, bincode::config::standard())?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Eliminar un key en la transacción
    pub fn remove<K: AsRef<[u8]>>(&self, key: K) -> ConflictableTransactionResult<(), Box<dyn std::error::Error>> {
        self.tree.remove(key)?;
        Ok(())
    }

    /// Iterar sobre los elementos en la transacción
    pub fn iter(&self) -> impl Iterator<Item = ConflictableTransactionResult<(IVec, IVec), Box<dyn std::error::Error>>> + '_ {
        self.tree.iter().map(|item| {
            item.map_err(|e| e.into())
        })
    }
}