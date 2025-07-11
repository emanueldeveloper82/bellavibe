// src/produtos/produtos_router.rs

use actix_web::{get, post, web, HttpResponse, Responder};
//use sqlx::{query_as, Row, Pool, Postgres}; // Pool e Postgres ainda são necessários para verificar produtos
use sqlx::{query_as, Row}; 
use serde_json;
//use bigdecimal::BigDecimal;
use std::sync::RwLock;

// Importa as structs específicas de produtos
use super::produtos_structs::{
    NovoProduto,
    Produto,
    ProdutoResponse,
    Carrinho    
};

// Importa ItemVenda e GenericResponse do novo módulo de vendas
// Usamos 'crate::' para importar diretamente do módulo 'vendas' no nível raiz do crate.
use crate::vendas::vendas_structs::{ItemVenda, GenericResponse}; // <-- Importação de ItemVenda corrigida aqui

// Importa o AppState do módulo raiz (main.rs)
use crate::AppState;

/// Rota para buscar todos os produtos no banco de dados.
/// Retorna uma GenericResponse com a lista de produtos.
#[get("/produtos")]
pub async fn buscar_produtos(data: web::Data<AppState>) -> impl Responder {
    let produtos_result = query_as::<_, Produto>("SELECT id, nome, descricao, preco, estoque FROM produtos")
        .fetch_all(&data.db_pool)
        .await;

    match produtos_result {
        Ok(produtos) => {
            let response_body: Vec<ProdutoResponse> = produtos.into_iter()
                .map(|p| ProdutoResponse {
                    id: p.id,
                    nome: p.nome,
                    descricao: p.descricao,
                    preco: p.preco,
                    estoque: p.estoque,
                })
                .collect();
            
            HttpResponse::Ok().json(GenericResponse {
                status: "success".to_string(),
                message: "Produtos listados com sucesso!".to_string(),
                body: Some(response_body),
            })
        },
        Err(e) => {
            eprintln!("Erro ao buscar produtos: {:?}", e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao buscar produtos".to_string(),
                body: None,
            })
        }
    }
}

/// Rota para inserir um novo produto no banco de dados.
/// Retorna uma GenericResponse com o ID do produto criado.
#[post("/produtos")]
pub async fn cadastrar_produto(
    data: web::Data<AppState>,
    item: web::Json<NovoProduto>,
) -> HttpResponse {
    let result = sqlx::query(
        "INSERT INTO produtos (nome, descricao, preco, estoque) VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(&item.nome)
    .bind(&item.descricao)
    .bind(&item.preco)
    .bind(item.estoque)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(row) => {
            match row.try_get::<i32, &str>("id") {
                Ok(id) => {
                    HttpResponse::Ok().json(GenericResponse {
                        status: "success".to_string(),
                        message: format!("Produto cadastrado com sucesso! ID: {}", id),
                        body: Some(serde_json::json!({ "id": id })),
                    })
                },
                Err(e) => {
                    eprintln!("Erro ao obter id do novo produto: {:?}", e);
                    HttpResponse::InternalServerError().json(GenericResponse::<()>{
                        status: "error".to_string(),
                        message: "Erro ao processar resposta do cadastro".to_string(),
                        body: None,
                    })
                }
            }
        }
        Err(e) => {
            eprintln!("Erro ao inserir produto: {:?}", e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao inserir produto".to_string(),
                body: None,
            })
        }
    }
}

// --- Rotas para a funcionalidade de Sacola (permanecem aqui por enquanto) ---

/// Rota para adicionar um item à sacola de compras.
/// Recebe um ItemVenda no corpo da requisição.
#[post("/sacola/adicionar")]
pub async fn adicionar_item_sacola(
    carrinho_data: web::Data<RwLock<Carrinho>>, // Acesso ao estado da sacola
    item_venda: web::Json<ItemVenda>,
    data: web::Data<AppState>, // Necessário para verificar o produto no DB
) -> HttpResponse {
    // Verifica se o produto existe no banco de dados
    let produto_exists = sqlx::query_as::<_, Produto>(
        "SELECT id, nome, descricao, preco, estoque FROM produtos WHERE id = $1"
    )
    .bind(item_venda.produto_id)
    .fetch_optional(&data.db_pool)
    .await;

    match produto_exists {
        Ok(Some(_)) => {
            let mut carrinho = carrinho_data.write().unwrap(); // Obtém um lock de escrita

            // Verifica se o produto já existe na sacola
            let mut found = false;
            for item_in_cart in carrinho.itens.iter_mut() {
                if item_in_cart.produto_id == item_venda.produto_id {
                    item_in_cart.quantidade += item_venda.quantidade; // Soma a quantidade
                    found = true;
                    break;
                }
            }

            if !found {
                // Se o produto não foi encontrado, adiciona como um novo item
                carrinho.itens.push(item_venda.into_inner());
            }

            HttpResponse::Ok().json(GenericResponse::<()>{
                status: "success".to_string(),
                message: "Item adicionado/atualizado na sacola com sucesso!".to_string(),
                body: None,
            })
        },
        Ok(None) => {
            HttpResponse::BadRequest().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: format!("Produto com ID {} não encontrado para adicionar à sacola.", item_venda.produto_id),
                body: None,
            })
        },
        Err(e) => {
            eprintln!("Erro ao verificar produto para adicionar à sacola: {:?}", e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro interno ao verificar produto".to_string(),
                body: None,
            })
        }
    }
}

/// Rota para visualizar o conteúdo atual da sacola de compras.
#[get("/sacola")]
pub async fn ver_sacola(carrinho_data: web::Data<RwLock<Carrinho>>) -> HttpResponse {
    let carrinho = carrinho_data.read().unwrap(); // Obtém um lock de leitura
    
    HttpResponse::Ok().json(GenericResponse {
        status: "success".to_string(),
        message: "Conteúdo da sacola".to_string(),
        body: Some(carrinho.itens.clone()), // Clona os itens para a resposta
    })
}
