// src/produtos/produtos_router.rs

use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use sqlx::{query_as, query, Row}; // Importa 'query' também para UPDATE/DELETE
use serde_json;

// Importa as structs específicas de produtos
use super::produtos_structs::{
    NovoProduto,    
    ProdutoResponse,    
    ProdutoRawData,
};

// Importa GenericResponse do novo módulo shared_structs
use crate::shared::shared_structs::GenericResponse;

// Importa o AppState do módulo raiz (main.rs)
use crate::AppState;

// Importa o extrator de autenticação
use crate::usuarios::auth_middleware::AuthenticatedUser; 


/// Rota para buscar todos os produtos no banco de dados.
/// Retorna uma GenericResponse com a lista de produtos, incluindo o nome da categoria.
#[get("/produtos")]
pub async fn buscar_produtos(
    data: web::Data<AppState>, 
    auth_user: AuthenticatedUser,
) -> impl Responder {
    

    //Gamb pra remover o dead_code ao subir a aplicação. Vou pensar em algo e remover esse if-else.
    if auth_user.user_id > 0 && !auth_user.user_name.is_empty() && !auth_user.user_email.is_empty() {
        println!("Usuário autenticado: buscar_produtos()"); 
    } else {
        // Opcional: logar um aviso se os dados do usuário estiverem incompletos,
        // embora a validação do JWT já garanta a presença de sub, name e email.
        eprintln!("Aviso: Dados do usuário autenticado incompletos ou inválidos.");
    }
    
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
    .bind(item.categoria_id) 
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


