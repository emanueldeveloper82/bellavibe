// src/vendas/vendas_structs.rs

use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

/// Estrutura para representar um item individual dentro de uma venda ou sacola.
/// É usada tanto para adicionar itens à sacola quanto para processar a venda.
#[derive(Deserialize, Serialize, Clone)]
pub struct ItemVenda {
    pub produto_id: i32,
    pub quantidade: i32,
}

/// Estrutura para a resposta de sucesso da venda.
/// Contém o valor total da compra e uma mensagem de confirmação.
#[derive(Serialize)]
pub struct VendaResponse {
    pub total_compra: BigDecimal,
    pub mensagem: String,
}

