// src/produtos/produtos_structs.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use bigdecimal::BigDecimal; // Importa BigDecimal

/// Estrutura para receber dados do novo produto na requisição POST
#[derive(Deserialize)]
pub struct NovoProduto {
    pub nome: String,
    pub descricao: String,
    pub preco: BigDecimal,
    pub estoque: i32,
    pub categoria_id: i32,
}

/// Estrutura que representa um produto no banco de dados
/// Deriva FromRow para mapeamento direto de resultados de query SQL
#[derive(Serialize, FromRow)]
pub struct Produto {
    pub id: i32,
    pub nome: String,
    pub descricao: String,
    pub preco: BigDecimal,
    pub estoque: i32,
    pub categoria_id: i32, 
}

/// Estrutura para a resposta da API ao buscar produtos
/// Usada para serializar os dados do produto para JSON
#[derive(Serialize)]
pub struct ProdutoResponse {
    pub id: i32,
    pub nome: String,
    pub descricao: String,
    pub preco: BigDecimal,
    pub estoque: i32,
    pub categoria_id: i32,     
    pub categoria_nome: String,
}


/// Estrutura auxiliar para mapear diretamente o resultado da query SQL com JOIN.
/// Contém todos os campos selecionados, incluindo o nome da categoria.
#[derive(FromRow)]
pub struct ProdutoRawData {
    pub id: i32,
    pub nome: String,
    pub descricao: String,
    pub preco: BigDecimal,
    pub estoque: i32,
    pub categoria_id: i32,
    pub categoria_nome: String, // Corresponde a 'c.nome AS categoria_nome' na query
}
