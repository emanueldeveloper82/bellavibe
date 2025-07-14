// src/categorias/categoria_router.rs

use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use sqlx::{query_as, query, Row};

// Importa as structs de categoria
use super::categoria_structs::{Categoria, NovaCategoria};

use crate::vendas::vendas_structs::GenericResponse;

// Importa o AppState do módulo raiz (main.rs)
use crate::AppState;

/// Rota para cadastrar uma nova categoria.
#[post("/categorias")]
pub async fn cadastrar_categoria(
    data: web::Data<AppState>,
    item: web::Json<NovaCategoria>,
) -> HttpResponse {
    let result = query(        
        "INSERT INTO categorias (nome, parent_id) VALUES ($1, $2) RETURNING id"
    )
    .bind(&item.nome)    
    .bind(item.parent_id)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(row) => {
            match row.try_get::<i32, &str>("id") {
                Ok(id) => HttpResponse::Ok().json(GenericResponse {
                    status: "success".to_string(),
                    message: format!("Categoria cadastrada com sucesso! ID: {}", id),
                    body: Some(serde_json::json!({ "id": id })),
                }),
                Err(e) => {
                    eprintln!("Erro ao obter id da nova categoria: {:?}", e);
                    HttpResponse::InternalServerError().json(GenericResponse::<()>{
                        status: "error".to_string(),
                        message: "Erro ao processar resposta do cadastro da categoria".to_string(),
                        body: None,
                    })
                }
            }
        }
        Err(e) => {
            eprintln!("Erro ao inserir categoria: {:?}", e);            
            let error_message = if e.to_string().contains("foreign key constraint") {
                "Erro ao inserir categoria: parent_id inválido. Verifique o ID da categoria pai.".to_string()
            } else {
                "Erro ao inserir categoria.".to_string()
            };
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: error_message,
                body: None,
            })
        }
    }
}

/// Rota para buscar todas as categorias.
#[get("/categorias")]
pub async fn buscar_categorias(data: web::Data<AppState>) -> impl Responder {
    let categorias_result = query_as::<_, Categoria>("SELECT id, nome, parent_id FROM categorias ORDER BY id")
        .fetch_all(&data.db_pool)
        .await;

    match categorias_result {
        Ok(categorias) => {
            HttpResponse::Ok().json(GenericResponse {
                status: "success".to_string(),
                message: "Categorias listadas com sucesso!".to_string(),
                body: Some(categorias),
            })
        },
        Err(e) => {
            eprintln!("Erro ao buscar categorias: {:?}", e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao buscar categorias".to_string(),
                body: None,
            })
        }
    }
}

/// Rota para buscar uma categoria por ID.
#[get("/categorias/{id}")]
pub async fn buscar_categoria_por_id(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> HttpResponse {
    let id = path.into_inner();
    let categoria_result = query_as::<_, Categoria>("SELECT id, nome, parent_id FROM categorias WHERE id = $1")
        .bind(id)
        .fetch_optional(&data.db_pool)
        .await;

    match categoria_result {
        Ok(Some(categoria)) => HttpResponse::Ok().json(GenericResponse {
            status: "success".to_string(),
            message: format!("Categoria com ID {} encontrada.", id),
            body: Some(categoria),
        }),
        Ok(None) => HttpResponse::NotFound().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: format!("Categoria com ID {} não encontrada.", id),
            body: None,
        }),
        Err(e) => {
            eprintln!("Erro ao buscar categoria por ID {}: {:?}", id, e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao buscar categoria".to_string(),
                body: None,
            })
        }
    }
}

/// Rota para atualizar uma categoria existente.
#[put("/categorias/{id}")]
pub async fn atualizar_categoria(
    data: web::Data<AppState>,
    path: web::Path<i32>,
    item: web::Json<NovaCategoria>,
) -> HttpResponse {
    let id = path.into_inner();
    let result = query(
        "UPDATE categorias SET nome = $1, parent_id = $2 WHERE id = $3"
    )
    .bind(&item.nome)
    .bind(item.parent_id) 
    .bind(id)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                HttpResponse::Ok().json(GenericResponse::<()>{
                    status: "success".to_string(),
                    message: format!("Categoria com ID {} atualizada com sucesso.", id),
                    body: None,
                })
            } else {
                HttpResponse::NotFound().json(GenericResponse::<()>{
                    status: "error".to_string(),
                    message: format!("Categoria com ID {} não encontrada para atualização.", id),
                    body: None,
                })
            }
        },
        Err(e) => {
            eprintln!("Erro ao atualizar categoria com ID {}: {:?}", id, e);
            // Melhorar a mensagem de erro se for uma violação de FK (parent_id inválido)
            let error_message = if e.to_string().contains("foreign key constraint") {
                "Erro ao atualizar categoria: parent_id inválido. Verifique o ID da categoria pai.".to_string()
            } else {
                "Erro ao atualizar categoria.".to_string()
            };
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: error_message,
                body: None,
            })
        }
    }
}

/// Rota para deletar uma categoria.
#[delete("/categorias/{id}")]
pub async fn deletar_categoria(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> HttpResponse {
    let id = path.into_inner();
    let result = query("DELETE FROM categorias WHERE id = $1")
        .bind(id)
        .execute(&data.db_pool)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                HttpResponse::Ok().json(GenericResponse::<()>{
                    status: "success".to_string(),
                    message: format!("Categoria com ID {} deletada com sucesso.", id),
                    body: None,
                })
            } else {
                HttpResponse::NotFound().json(GenericResponse::<()>{
                    status: "error".to_string(),
                    message: format!("Categoria com ID {} não encontrada para exclusão.", id),
                    body: None,
                })
            }
        },
        Err(e) => {
            eprintln!("Erro ao deletar categoria com ID {}: {:?}", id, e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao deletar categoria".to_string(),
                body: None,
            })
        }
    }
}
