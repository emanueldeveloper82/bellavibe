// src/categorias/categoria_structs.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Estrutura para receber dados de uma nova categoria na requisição POST/PUT
#[derive(Deserialize)]
pub struct NovaCategoria {
    pub nome: String,
    pub parent_id: Option<i32>,
}

/// Estrutura que representa uma categoria no banco de dados
#[derive(Serialize, FromRow)]
pub struct Categoria {
    pub id: i32,
    pub nome: String,
    pub parent_id: Option<i32>,
}

// Re-exporta GenericResponse para que possa ser facilmente usada dentro do módulo categorias
//pub use crate::vendas::vendas_structs::GenericResponse;
