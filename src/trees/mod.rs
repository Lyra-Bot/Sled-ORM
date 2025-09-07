use std::error::Error;

use crate::{bincode_get_config,  Tree};
use serde::{Serialize, Deserialize};
use bincode;
use sled::{transaction::{ConflictableTransactionResult, TransactionResult, TransactionalTree}, IVec};

impl Tree {
    pub fn insert<K, V>(&self, key: K, value: &V) -> Result<(), Box<dyn std::error::Error>>
    where
        K: AsRef<[u8]>,
        V: Serialize,
    {
        // Usar la función correcta de bincode::serde
        let serialized = bincode::serde::encode_to_vec(value, bincode_get_config())?;
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
            let (value, _) = bincode::serde::decode_from_slice(&ivec, bincode_get_config())?;
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
            let (deserialized, _) = bincode::serde::decode_from_slice(&value, bincode_get_config())?;
            
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

    pub fn all<V>(&self) -> Result<Vec<V>, Box<dyn Error>>
    where
        V: for<'de> Deserialize<'de>,
    {
        let mut results = Vec::new();
        
        for item in self.tree.iter() {
            let (_, value) = item?;
            let (deserialized, _) = bincode::serde::decode_from_slice(&value, bincode_get_config())?;
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
        E: From<Box<dyn Error>> + From<String>,
    {
        self.tree.transaction(f)
    }

    
}

