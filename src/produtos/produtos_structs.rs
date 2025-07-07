// src/produtos/produtos_structs.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use bigdecimal::BigDecimal; // Importe BigDecimal

/// Estrutura para receber dados do novo produto na requisição POST
#[derive(Deserialize)]
pub struct NovoProduto {
    pub nome: String,
    pub descricao: String,
    pub preco: BigDecimal,
    pub estoque: i32,
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
}

/// Estrutura para representar um item individual dentro de uma venda
// Adicionado Serialize para poder retornar na resposta, se necessário
#[derive(Deserialize, Serialize)] 
pub struct ItemVenda {
    pub produto_id: i32,
    pub quantidade: i32,
}

/// Estrutura para a requisição de venda
#[derive(Deserialize)]
pub struct VendaRequest {
    pub itens: Vec<ItemVenda>,
}

/// Estrutura para a resposta de sucesso da venda
#[derive(Serialize)]
pub struct VendaResponse {
    pub total_compra: BigDecimal,
    pub mensagem: String,
}