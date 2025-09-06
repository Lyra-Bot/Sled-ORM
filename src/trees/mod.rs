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

    // Los demás métodos (delete, update, all) permanecen similares
    // pero asegúrate de usar IVec::from() para la serialización
}