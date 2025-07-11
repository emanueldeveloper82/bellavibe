// src/produtos/produtos_structs.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use bigdecimal::BigDecimal; 


// Importa ItemVenda do módulo de vendas, pois Carrinho ainda depende dela aqui
// O 'crate::' garante que estamos importando do módulo 'vendas' no nível raiz do crate.
use crate::vendas::vendas_structs::ItemVenda;


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

/// Estrutura para representar a sacola de compras em memória (para este MVP)
#[derive(Default)] // Permite criar uma instância padrão (com vetor vazio)
pub struct Carrinho {
    pub itens: Vec<ItemVenda>,
}