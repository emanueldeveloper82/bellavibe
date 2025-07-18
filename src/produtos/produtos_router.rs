// src/produtos/produtos_router.rs

use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use sqlx::{query_as, query, Row}; // Importa 'query' também para UPDATE/DELETE
use serde_json;
use std::sync::RwLock;

// Importa as structs específicas de produtos
use super::produtos_structs::{
    NovoProduto,
    Produto,
    ProdutoResponse,
    Carrinho,
    ProdutoRawData,
};

// Importa ItemVenda do módulo de vendas
use crate::vendas::vendas_structs::ItemVenda;
// Importa GenericResponse do novo módulo shared_structs
use crate::shared::shared_structs::GenericResponse;

// Importa o AppState do módulo raiz (main.rs)
use crate::AppState;

/// Rota para buscar todos os produtos no banco de dados.
/// Retorna uma GenericResponse com a lista de produtos, incluindo o nome da categoria.
#[get("/produtos")]
pub async fn buscar_produtos(data: web::Data<AppState>) -> impl Responder {
    // A consulta agora faz um JOIN com a tabela 'categorias' para obter o nome da categoria.
    // Usamos ProdutoRawData para mapear o resultado completo do JOIN.
    let produtos_result = query_as::<_, ProdutoRawData>(
        r#"
        SELECT 
            p.id, p.nome, p.descricao, p.preco, p.estoque, p.categoria_id,
            c.nome AS categoria_nome
        FROM produtos p
        JOIN categorias c ON p.categoria_id = c.id
        ORDER BY p.id
        "#
    )
    .fetch_all(&data.db_pool)
    .await;

    match produtos_result {
        Ok(produtos_raw) => {
            let response_body: Vec<ProdutoResponse> = produtos_raw.into_iter()
                .map(|p_raw| ProdutoResponse { // Mapeia ProdutoRawData para ProdutoResponse
                    id: p_raw.id,
                    nome: p_raw.nome,
                    descricao: p_raw.descricao,
                    preco: p_raw.preco,
                    estoque: p_raw.estoque,
                    categoria_id: p_raw.categoria_id,
                    categoria_nome: p_raw.categoria_nome, // Agora acessa diretamente de p_raw
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

/// Rota para buscar um produto específico por ID.
/// Retorna uma GenericResponse com os detalhes do produto, incluindo o nome da categoria.
#[get("/produtos/{id}")]
pub async fn buscar_produto_por_id(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> HttpResponse {
    let id = path.into_inner();
    let produto_result = query_as::<_, ProdutoRawData>(
        r#"
        SELECT 
            p.id, p.nome, p.descricao, p.preco, p.estoque, p.categoria_id,
            c.nome AS categoria_nome
        FROM produtos p
        JOIN categorias c ON p.categoria_id = c.id
        WHERE p.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&data.db_pool)
    .await;

    match produto_result {
        Ok(Some(p_raw)) => {
            let response_body = ProdutoResponse {
                id: p_raw.id,
                nome: p_raw.nome,
                descricao: p_raw.descricao,
                preco: p_raw.preco,
                estoque: p_raw.estoque,
                categoria_id: p_raw.categoria_id,
                categoria_nome: p_raw.categoria_nome,
            };
            HttpResponse::Ok().json(GenericResponse {
                status: "success".to_string(),
                message: format!("Produto com ID {} encontrado.", id),
                body: Some(response_body),
            })
        },
        Ok(None) => HttpResponse::NotFound().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: format!("Produto com ID {} não encontrado.", id),
            body: None,
        }),
        Err(e) => {
            eprintln!("Erro ao buscar produto por ID {}: {:?}", id, e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao buscar produto por ID".to_string(),
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
    // A query SQL agora inclui o categoria_id
    let result = sqlx::query(
        "INSERT INTO produtos (nome, descricao, preco, estoque, categoria_id) VALUES ($1, $2, $3, $4, $5) RETURNING id"
    )
    .bind(&item.nome)
    .bind(&item.descricao)
    .bind(&item.preco)
    .bind(item.estoque)
    .bind(item.categoria_id) // <-- Binda o ID da categoria (nome do campo ajustado)
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
            // Melhorar a mensagem de erro para o cliente, se for uma violação de FK
            let error_message = if e.to_string().contains("foreign key constraint") {
                "Erro ao inserir produto: Categoria não encontrada. Verifique o categoria_id.".to_string()
            } else {
                "Erro ao inserir produto.".to_string()
            };
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: error_message,
                body: None,
            })
        }
    }
}

/// Rota para atualizar um produto existente por ID.
/// Retorna uma GenericResponse de sucesso ou erro.
#[put("/produtos/{id}")]
pub async fn atualizar_produto(
    data: web::Data<AppState>,
    path: web::Path<i32>,
    item: web::Json<NovoProduto>,
) -> HttpResponse {
    let id = path.into_inner();
    let result = query(
        "UPDATE produtos SET nome = $1, descricao = $2, preco = $3, estoque = $4, categoria_id = $5 WHERE id = $6"
    )
    .bind(&item.nome)
    .bind(&item.descricao)
    .bind(&item.preco)
    .bind(item.estoque)
    .bind(item.categoria_id)
    .bind(id)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                HttpResponse::Ok().json(GenericResponse::<()>{
                    status: "success".to_string(),
                    message: format!("Produto com ID {} atualizado com sucesso.", id),
                    body: None,
                })
            } else {
                HttpResponse::NotFound().json(GenericResponse::<()>{
                    status: "error".to_string(),
                    message: format!("Produto com ID {} não encontrado para atualização.", id),
                    body: None,
                })
            }
        },
        Err(e) => {
            eprintln!("Erro ao atualizar produto com ID {}: {:?}", id, e);
            let error_message = if e.to_string().contains("foreign key constraint") {
                "Erro ao atualizar produto: Categoria não encontrada. Verifique o categoria_id.".to_string()
            } else {
                "Erro ao atualizar produto.".to_string()
            };
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: error_message,
                body: None,
            })
        }
    }
}

/// Rota para deletar um produto por ID.
/// Retorna uma GenericResponse de sucesso ou erro.
#[delete("/produtos/{id}")]
pub async fn deletar_produto(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> HttpResponse {
    let id = path.into_inner();
    let result = query("DELETE FROM produtos WHERE id = $1")
        .bind(id)
        .execute(&data.db_pool)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                HttpResponse::Ok().json(GenericResponse::<()>{
                    status: "success".to_string(),
                    message: format!("Produto com ID {} deletado com sucesso.", id),
                    body: None,
                })
            } else {
                HttpResponse::NotFound().json(GenericResponse::<()>{
                    status: "error".to_string(),
                    message: format!("Produto com ID {} não encontrado para exclusão.", id),
                    body: None,
                })
            }
        },
        Err(e) => {
            eprintln!("Erro ao deletar produto com ID {}: {:?}", id, e);
            // Adicionar tratamento para erro de chave estrangeira se o produto estiver associado a vendas/carrinhos persistentes
            let error_message = if e.to_string().contains("foreign key constraint") {
                "Não é possível deletar o produto: ele está associado a vendas ou carrinhos existentes.".to_string()
            } else {
                "Erro ao deletar produto.".to_string()
            };
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: error_message,
                body: None,
            })
        }
    }
}

// --- Rotas para a funcionalidade de Sacola ---

/// Rota para adicionar um item à sacola de compras.
/// Recebe um ItemVenda no corpo da requisição.
#[post("/sacola/adicionar")]
pub async fn adicionar_item_sacola(
    carrinho_data: web::Data<RwLock<Carrinho>>, // Acesso ao estado da sacola
    item_venda: web::Json<ItemVenda>,
    data: web::Data<AppState>, // Necessário para verificar o produto no DB
) -> HttpResponse {
    // Verifica se o produto existe no banco de dados
    // Inclui categoria_id na seleção (nome do campo ajustado)
    let produto_exists = sqlx::query_as::<_, Produto>(
        "SELECT id, nome, descricao, preco, estoque, categoria_id FROM produtos WHERE id = $1" 
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
