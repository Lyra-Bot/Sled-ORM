
#[macro_export]
macro_rules! model {
    ($name:ident { $($field:ident: $type:ty),* }) => {
        #[derive(Serialize, Deserialize, Debug)]
        pub struct $name {
            $(pub $field: $type),*
        }

        impl $name {
            pub fn save(&self, tree: &Tree) -> Result<(), Box<dyn std::error::Error>> {
                // Generar ID automático si no existe
                tree.insert(self.id().as_bytes(), self)
            }

            pub fn id(&self) -> String {
                // Implementar lógica para generar ID único
                format!("{}:{}", stringify!($name).to_lowercase(), uuid::Uuid::new_v4())
            }
        }
    };
}