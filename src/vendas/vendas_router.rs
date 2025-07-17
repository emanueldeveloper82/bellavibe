// src/vendas/vendas_router.rs

use actix_web::{post, web, HttpResponse};
//use sqlx::{query_as, Row, Pool, Postgres, Transaction};
use bigdecimal::BigDecimal;
use std::sync::RwLock;

// Importa as structs necessárias do módulo de produtos (para Produto)
use crate::produtos::produtos_structs::Produto;
// Importa as structs de vendas (ItemVenda, VendaResponse)
use super::vendas_structs::{VendaResponse};
// Importa o AppState do módulo raiz (main.rs)
use crate::AppState;
// Importa o Carrinho do módulo de produtos (ainda está lá)
use crate::produtos::produtos_structs::Carrinho;
// Importa GenericResponse do novo módulo shared_structs
use crate::shared::shared_structs::GenericResponse;


/// Rota para realizar uma venda de produtos, consumindo itens da sacola.
///
/// Esta função orquestra o processo de venda, garantindo a atomicidade das operações
/// de verificação de estoque, cálculo do total e atualização do estoque através de uma transação de banco de dados.
///
/// Passos:
/// 1. Obtém os itens da sacola e a limpa.
/// 2. Inicia uma transação no banco de dados.
/// 3. Para cada item na sacola:
///    a. Busca o produto e o bloqueia para atualização (`FOR UPDATE`).
///    b. Verifica a disponibilidade de estoque.
///    c. Calcula o subtotal e adiciona ao total da compra.
///    d. Decrementa o estoque do produto.
/// 4. Se todas as operações forem bem-sucedidas, comita a transação.
/// 5. Retorna o valor total da compra em caso de sucesso ou uma mensagem de erro.
#[post("/venda")]
pub async fn realizar_venda(
    data: web::Data<AppState>,
    carrinho_data: web::Data<RwLock<Carrinho>>, // Acesso ao estado da sacola
) -> HttpResponse {
    // Pega os itens da sacola e limpa-a. Isso é feito dentro de um bloco para liberar o lock de escrita rapidamente.
    let itens_venda = {
        let mut carrinho = carrinho_data.write().unwrap();
        if carrinho.itens.is_empty() {
            return HttpResponse::BadRequest().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "A sacola está vazia. Adicione itens antes de realizar a venda.".to_string(),
                body: None,
            });
        }
        std::mem::take(&mut carrinho.itens) // Pega os itens e deixa o vetor vazio
    };

    // Inicia uma transação no banco de dados para garantir atomicidade
    let mut transaction = match data.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Erro ao iniciar transação: {:?}", e);
            return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro interno ao processar venda".to_string(),
                body: None,
            });
        }
    };

    let mut total_compra = BigDecimal::from(0); // Inicializa o total da compra com 0

    // Itera sobre cada item na sacola
    for item in itens_venda.iter() {
        // 1. Busca o produto no banco de dados para verificar estoque e preço
        // FOR UPDATE bloqueia a linha para evitar race conditions em ambientes multi-usuário
        let produto_result = sqlx::query_as::<_, Produto>(
            "SELECT id, nome, descricao, preco, estoque FROM produtos WHERE id = $1 FOR UPDATE"
        )
        .bind(item.produto_id)
        .fetch_optional(&mut *transaction) // Usa a transação para a consulta
        .await;

        let produto = match produto_result {
            Ok(Some(p)) => p,
            Ok(None) => {
                eprintln!("Produto com ID {} não encontrado durante a venda.", item.produto_id);
                let _ = transaction.rollback().await;
                return HttpResponse::BadRequest().json(GenericResponse::<()>{
                    status: "error".to_string(),
                    message: format!("Produto com ID {} não encontrado para venda.", item.produto_id),
                    body: None,
                });
            },
            Err(e) => {
                eprintln!("Erro ao buscar produto {}: {:?}", item.produto_id, e);
                let _ = transaction.rollback().await;
                return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                    status: "error".to_string(),
                    message: "Erro ao buscar produto para venda".to_string(),
                    body: None,
                });
            }
        };

        // 2. Verifica se há estoque suficiente
        if produto.estoque < item.quantidade {
            eprintln!("Estoque insuficiente para o produto {}. Disponível: {}, Solicitado: {}",
                      produto.nome, produto.estoque, item.quantidade);
            let _ = transaction.rollback().await;
            return HttpResponse::BadRequest().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: format!("Estoque insuficiente para o produto {}.", produto.nome),
                body: None,
            });
        }

        // Calcula o subtotal para o item e adiciona ao total da compra
        let quantidade_bigdecimal = BigDecimal::from(item.quantidade);
        let subtotal = &produto.preco * &quantidade_bigdecimal;
        total_compra += subtotal;

        // 3. Decrementa o estoque do produto
        let novo_estoque = produto.estoque - item.quantidade;
        let update_result = sqlx::query(
            "UPDATE produtos SET estoque = $1 WHERE id = $2"
        )
        .bind(novo_estoque)
        .bind(item.produto_id)
        .execute(&mut *transaction)
        .await;

        if let Err(e) = update_result {
            eprintln!("Erro ao atualizar estoque do produto {}: {:?}", produto.nome, e);
            let _ = transaction.rollback().await;
            return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao atualizar estoque durante a venda".to_string(),
                body: None,
            });
        }
    }

    // Se todas as operações foram bem-sucedidas, comita a transação
    if let Err(e) = transaction.commit().await {
        eprintln!("Erro ao comitar transação: {:?}", e);
        return HttpResponse::InternalServerError().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: "Erro interno ao finalizar venda".to_string(),
            body: None,
        });
    }

    // Retorna a resposta de sucesso com o total da compra
    HttpResponse::Ok().json(GenericResponse {
        status: "success".to_string(),
        message: "Venda realizada com sucesso!".to_string(),
        body: Some(VendaResponse {
            total_compra,
            mensagem: "Venda processada e sacola limpa.".to_string(),
        }),
    })
}
