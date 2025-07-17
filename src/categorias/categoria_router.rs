// src/categorias/categoria_router.rs

use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use sqlx::{query_as, query, Row};

// Importa as structs de categoria
use super::categoria_structs::{Categoria, NovaCategoria};
// Importa GenericResponse do novo módulo shared_structs
use crate::shared::shared_structs::GenericResponse;

// Importa o AppState do módulo raiz (main.rs)
use crate::AppState;


// --- Rotas para SESSÕES (Categorias Pai) ---

/// Rota para cadastrar uma nova SESSÃO (Categoria Pai).
/// O campo `parent_id` será obrigatoriamente NULL para sessões.
#[post("/sessoes")]
pub async fn cadastrar_sessao(
    data: web::Data<AppState>,
    item: web::Json<NovaCategoria>, // Reutiliza NovaCategoria, mas parent_id será ignorado/forçado a NULL
) -> HttpResponse {
    let result = query(
        "INSERT INTO categorias (nome, parent_id) VALUES ($1, NULL) RETURNING id" // Força parent_id para NULL
    )
    .bind(&item.nome)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(row) => {
            match row.try_get::<i32, &str>("id") {
                Ok(id) => HttpResponse::Ok().json(GenericResponse {
                    status: "success".to_string(),
                    message: format!("Sessão cadastrada com sucesso! ID: {}", id),
                    body: Some(serde_json::json!({ "id": id })),
                }),
                Err(e) => {
                    eprintln!("Erro ao obter id da nova sessão: {:?}", e);
                    HttpResponse::InternalServerError().json(GenericResponse::<()>{
                        status: "error".to_string(),
                        message: "Erro ao processar resposta do cadastro da sessão".to_string(),
                        body: None,
                    })
                }
            }
        }
        Err(e) => {
            eprintln!("Erro ao inserir sessão: {:?}", e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao inserir sessão".to_string(),
                body: None,
            })
        }
    }
}

/// Rota para buscar todas as SESSÕES (Categorias Pai).
/// Retorna apenas as categorias onde `parent_id` é NULL.
#[get("/sessoes")]
pub async fn buscar_sessoes(data: web::Data<AppState>) -> impl Responder {
    let categorias_result = query_as::<_, Categoria>("SELECT id, nome, parent_id FROM categorias WHERE parent_id IS NULL ORDER BY id")
        .fetch_all(&data.db_pool)
        .await;

    match categorias_result {
        Ok(sessoes) => {
            HttpResponse::Ok().json(GenericResponse {
                status: "success".to_string(),
                message: "Sessões listadas com sucesso!".to_string(),
                body: Some(sessoes),
            })
        },
        Err(e) => {
            eprintln!("Erro ao buscar sessões: {:?}", e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao buscar sessões".to_string(),
                body: None,
            })
        }
    }
}

/// Rota para buscar uma SESSÃO (Categoria Pai) por ID.
/// Retorna apenas a sessão se ela existir e tiver `parent_id` NULL.
#[get("/sessoes/{id}")]
pub async fn buscar_sessao_por_id(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> HttpResponse {
    let id = path.into_inner();
    let sessao_result = query_as::<_, Categoria>("SELECT id, nome, parent_id FROM categorias WHERE id = $1 AND parent_id IS NULL")
        .bind(id)
        .fetch_optional(&data.db_pool)
        .await;

    match sessao_result {
        Ok(Some(sessao)) => HttpResponse::Ok().json(GenericResponse {
            status: "success".to_string(),
            message: format!("Sessão com ID {} encontrada.", id),
            body: Some(sessao),
        }),
        Ok(None) => HttpResponse::NotFound().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: format!("Sessão com ID {} não encontrada ou não é uma sessão principal.", id),
            body: None,
        }),
        Err(e) => {
            eprintln!("Erro ao buscar sessão por ID {}: {:?}", id, e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao buscar sessão".to_string(),
                body: None,
            })
        }
    }
}

/// Rota para atualizar uma SESSÃO (Categoria Pai) existente.
/// Permite atualizar apenas o `nome`. O `parent_id` é mantido como NULL.
#[put("/sessoes/{id}")]
pub async fn atualizar_sessao(
    data: web::Data<AppState>,
    path: web::Path<i32>,
    item: web::Json<NovaCategoria>, // Reutiliza NovaCategoria, mas parent_id será ignorado
) -> HttpResponse {
    let id = path.into_inner();
    let result = query(
        "UPDATE categorias SET nome = $1 WHERE id = $2 AND parent_id IS NULL" // Garante que só atualiza sessões
    )
    .bind(&item.nome)
    .bind(id)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                HttpResponse::Ok().json(GenericResponse::<()>{
                    status: "success".to_string(),
                    message: format!("Sessão com ID {} atualizada com sucesso.", id),
                    body: None,
                })
            } else {
                HttpResponse::NotFound().json(GenericResponse::<()>{
                    status: "error".to_string(),
                    message: format!("Sessão com ID {} não encontrada ou não é uma sessão principal para atualização.", id),
                    body: None,
                })
            }
        },
        Err(e) => {
            eprintln!("Erro ao atualizar sessão com ID {}: {:?}", id, e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao atualizar sessão".to_string(),
                body: None,
            })
        }
    }
}

/// Rota para deletar uma SESSÃO (Categoria Pai).
/// Garante que apenas sessões (parent_id IS NULL) podem ser deletadas por esta rota.
/// Adiciona validação para impedir a exclusão de categorias filhas por este endpoint.
#[delete("/sessoes/{id}")]
pub async fn deletar_sessao(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> HttpResponse {
    let id = path.into_inner();

    // 1. Busca a categoria existente para verificar seu parent_id
    let existing_category_result = query_as::<_, Categoria>("SELECT id, nome, parent_id FROM categorias WHERE id = $1")
        .bind(id)
        .fetch_optional(&data.db_pool)
        .await;

    let existing_category = match existing_category_result {
        Ok(Some(cat)) => cat,
        Ok(None) => return HttpResponse::NotFound().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: format!("Sessão com ID {} não encontrada para exclusão.", id),
            body: None,
        }),
        Err(e) => {
            eprintln!("Erro ao buscar categoria existente para exclusão {}: {:?}", id, e);
            return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro interno ao buscar categoria para exclusão.".to_string(),
                body: None,
            });
        }
    };

    // 2. Validação: Se a categoria encontrada NÃO é uma sessão (parent_id IS NOT NULL),
    // retorna um erro.
    if existing_category.parent_id.is_some() {
        return HttpResponse::BadRequest().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: "Não é possível excluir uma categoria filha na rota de sessão. Use /categorias/{id} para isso.".to_string(),
            body: None,
        });
    }

    // 3. Procede com a exclusão da sessão (parent_id IS NULL)
    let result = query("DELETE FROM categorias WHERE id = $1 AND parent_id IS NULL") // Garante que só deleta sessões
        .bind(id)
        .execute(&data.db_pool)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                HttpResponse::Ok().json(GenericResponse::<()>{
                    status: "success".to_string(),
                    message: format!("Sessão com ID {} deletada com sucesso.", id),
                    body: None,
                })
            } else {
                // Esta parte pode ser redundante devido à verificação inicial, mas mantém a consistência
                HttpResponse::NotFound().json(GenericResponse::<()>{
                    status: "error".to_string(),
                    message: format!("Sessão com ID {} não encontrada para exclusão.", id),
                    body: None,
                })
            }
        },
        Err(e) => {
            eprintln!("Erro ao deletar sessão com ID {}: {:?}", id, e);
            // Adicionar tratamento para erro de chave estrangeira se houver categorias filhas
            let error_message = if e.to_string().contains("foreign key constraint") {
                "Não é possível deletar a sessão: existem categorias filhas associadas a ela.".to_string()
            } else {
                "Erro ao deletar sessão.".to_string()
            };
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: error_message,
                body: None,
            })
        }
    }
}



// --- Rotas para CATEGORIAS (Categorias Filhas/Subcategorias) ---

/// Rota para cadastrar uma nova CATEGORIA (Subcategoria).
/// O campo `parent_id` é OBRIGATÓRIO para categorias filhas.
#[post("/categorias")]
pub async fn cadastrar_categoria(
    data: web::Data<AppState>,
    item: web::Json<NovaCategoria>,
) -> HttpResponse {
    // Verifica se parent_id foi fornecido, pois é obrigatório para categorias filhas
    if item.parent_id.is_none() {
        return HttpResponse::BadRequest().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: "Para cadastrar uma categoria, o 'parent_id' é obrigatório.".to_string(),
            body: None,
        });
    }

    let result = query(
        "INSERT INTO categorias (nome, parent_id) VALUES ($1, $2) RETURNING id"
    )
    .bind(&item.nome)
    .bind(item.parent_id) // Binda o parent_id que deve ser fornecido
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

/// Rota para buscar CATEGORIAS FILHAS de uma SESSÃO específica.
/// Retorna categorias onde `parent_id` é igual ao ID da sessão fornecido.
#[get("/sessoes/{session_id}/categorias")]
pub async fn buscar_categorias_por_sessao(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> impl Responder {
    let session_id = path.into_inner();
    let categorias_result = query_as::<_, Categoria>("SELECT id, nome, parent_id FROM categorias WHERE parent_id = $1 ORDER BY id")
        .bind(session_id)
        .fetch_all(&data.db_pool)
        .await;

    match categorias_result {
        Ok(categorias) => {
            HttpResponse::Ok().json(GenericResponse {
                status: "success".to_string(),
                message: format!("Categorias da sessão {} listadas com sucesso!", session_id),
                body: Some(categorias),
            })
        },
        Err(e) => {
            eprintln!("Erro ao buscar categorias para sessão {}: {:?}", session_id, e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao buscar categorias por sessão".to_string(),
                body: None,
            })
        }
    }
}

// --- Rotas genéricas de Categoria (podem ser usadas para Sessões ou Categorias Filhas por ID) ---

/// Rota para buscar uma categoria (sessão ou filha) por ID.
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

/// Rota para atualizar uma categoria (sessão ou filha) existente.
/// Permite atualizar o `nome` e o `parent_id`.
/// Inclui validação para impedir que uma sessão se torne uma subcategoria
/// e que uma subcategoria se torne uma sessão.
#[put("/categorias/{id}")]
pub async fn atualizar_categoria(
    data: web::Data<AppState>,
    path: web::Path<i32>,
    item: web::Json<NovaCategoria>,
) -> HttpResponse {
    let id = path.into_inner();

    // 1. Busca a categoria existente para verificar seu parent_id atual
    let existing_category_result = query_as::<_, Categoria>("SELECT id, nome, parent_id FROM categorias WHERE id = $1")
        .bind(id)
        .fetch_optional(&data.db_pool)
        .await;

    let existing_category = match existing_category_result {
        Ok(Some(cat)) => cat,
        Ok(None) => return HttpResponse::NotFound().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: format!("Categoria com ID {} não encontrada para atualização.", id),
            body: None,
        }),
        Err(e) => {
            eprintln!("Erro ao buscar categoria existente para atualização {}: {:?}", id, e);
            return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro interno ao buscar categoria para atualização.".to_string(),
                body: None,
            });
        }
    };

    // 2. Validação 1: Se a categoria existente é uma sessão (parent_id IS NULL)
    // e a requisição tenta definir um parent_id (parent_id IS NOT NULL),
    // isso é um erro.
    if existing_category.parent_id.is_none() && item.parent_id.is_some() {
        return HttpResponse::BadRequest().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: "Uma sessão (categoria principal) não pode ser convertida em subcategoria.".to_string(),
            body: None,
        });
    }

    // 2. Validação 2: Se a categoria existente é uma subcategoria (parent_id IS NOT NULL)
    // e a requisição tenta definir o parent_id como NULL (tornando-a uma sessão),
    // isso é um erro.
    if existing_category.parent_id.is_some() && item.parent_id.is_none() {
        return HttpResponse::BadRequest().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: "Uma subcategoria não pode ser convertida em sessão principal.".to_string(),
            body: None,
        });
    }

    // 3. Procede com a atualização
    let result = query(
        "UPDATE categorias SET nome = $1, parent_id = $2 WHERE id = $3"
    )
    .bind(&item.nome)
    .bind(item.parent_id) // Binda o novo parent_id (pode ser NULL ou um ID válido)
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
                // Esta parte pode ser redundante devido às verificações iniciais, mas mantém a consistência
                HttpResponse::NotFound().json(GenericResponse::<()>{
                    status: "error".to_string(),
                    message: format!("Categoria com ID {} não encontrada para atualização.", id),
                    body: None,
                })
            }
        },
        Err(e) => {
            eprintln!("Erro ao atualizar categoria com ID {}: {:?}", id, e);
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

/// Rota para deletar uma categoria (sessão ou filha).
/// Esta rota pode deletar qualquer categoria pelo seu ID, mas com as devidas restrições de FK.
/// Adiciona validação para impedir a exclusão de sessões por este endpoint.
#[delete("/categorias/{id}")]
pub async fn deletar_categoria(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> HttpResponse {
    let id = path.into_inner();

    // 1. Busca a categoria existente para verificar seu parent_id
    let existing_category_result = query_as::<_, Categoria>("SELECT id, nome, parent_id FROM categorias WHERE id = $1")
        .bind(id)
        .fetch_optional(&data.db_pool)
        .await;

    let existing_category = match existing_category_result {
        Ok(Some(cat)) => cat,
        Ok(None) => return HttpResponse::NotFound().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: format!("Categoria com ID {} não encontrada para exclusão.", id),
            body: None,
        }),
        Err(e) => {
            eprintln!("Erro ao buscar categoria existente para exclusão {}: {:?}", id, e);
            return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro interno ao buscar categoria para exclusão.".to_string(),
                body: None,
            });
        }
    };

    // 2. Validação: Se a categoria encontrada É uma sessão (parent_id IS NULL),
    // retorna um erro, pois sessões devem ser deletadas pela rota específica.
    if existing_category.parent_id.is_none() {
        return HttpResponse::BadRequest().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: "Não é possível excluir uma sessão principal na rota de categorias. Use /sessoes/{id} para isso.".to_string(),
            body: None,
        });
    }

    // 3. Procede com a exclusão da categoria filha
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
            // Adicionar tratamento para erro de chave estrangeira se houver categorias filhas ou produtos associados
            let error_message = if e.to_string().contains("foreign key constraint") {
                "Não é possível deletar a categoria: existem subcategorias ou produtos associados a ela.".to_string()
            } else {
                "Erro ao deletar categoria.".to_string()
            };
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: error_message,
                body: None,
            })
        }
    }
}