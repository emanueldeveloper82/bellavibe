// src/usuarios/usuario_router.rs

use actix_web::{post, web, HttpResponse};
use sqlx::{query, query_as, Row};
use bcrypt::{hash, verify, DEFAULT_COST}; // Para hashing de senhas
use serde_json;

// Importa as structs do módulo de usuários
use super::usuario_structs::{NovoUsuario, LoginRequest, AuthResponse, Usuario};
// Importa GenericResponse do módulo shared_structs
use crate::shared::shared_structs::GenericResponse;
// Importa o AppState do módulo raiz (main.rs)
use crate::AppState;

/// Rota para cadastrar um novo usuário.
#[post("/usuarios/cadastro")]
pub async fn cadastrar_usuario(
    data: web::Data<AppState>,
    novo_usuario: web::Json<NovoUsuario>,
) -> HttpResponse {
    // 1. Verificar se o e-mail já está em uso
    let existing_user = query_as::<_, Usuario>("SELECT id, nome, email, senha_hash FROM usuarios WHERE email = $1")
        .bind(&novo_usuario.email)
        .fetch_optional(&data.db_pool)
        .await;

    match existing_user {
        Ok(Some(_)) => {
            return HttpResponse::BadRequest().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "E-mail já cadastrado.".to_string(),
                body: None,
            });
        },
        Err(e) => {
            eprintln!("Erro ao verificar e-mail existente: {:?}", e);
            return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro interno ao verificar e-mail.".to_string(),
                body: None,
            });
        },
        _ => {} // E-mail não encontrado, pode prosseguir
    }

    // 2. Hash da senha
    let hashed_password = match hash(&novo_usuario.senha, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Erro ao fazer hash da senha: {:?}", e);
            return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro interno ao processar senha.".to_string(),
                body: None,
            });
        }
    };

    // 3. Inserir o novo usuário no banco de dados
    let result = query(
        "INSERT INTO usuarios (nome, email, senha_hash) VALUES ($1, $2, $3) RETURNING id"
    )
    .bind(&novo_usuario.nome)
    .bind(&novo_usuario.email)
    .bind(&hashed_password)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(row) => {
            match row.try_get::<i32, &str>("id") {
                Ok(id) => HttpResponse::Ok().json(GenericResponse {
                    status: "success".to_string(),
                    message: format!("Usuário cadastrado com sucesso! ID: {}", id),
                    body: Some(serde_json::json!({ "id": id })),
                }),
                Err(e) => {
                    eprintln!("Erro ao obter id do novo usuário: {:?}", e);
                    HttpResponse::InternalServerError().json(GenericResponse::<()>{
                        status: "error".to_string(),
                        message: "Erro ao processar resposta do cadastro do usuário".to_string(),
                        body: None,
                    })
                }
            }
        }
        Err(e) => {
            eprintln!("Erro ao inserir usuário: {:?}", e);
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao inserir usuário".to_string(),
                body: None,
            })
        }
    }
}

/// Rota para login de usuário.
#[post("/usuarios/login")]
pub async fn login_usuario(
    data: web::Data<AppState>,
    login_request: web::Json<LoginRequest>,
) -> HttpResponse {
    // 1. Buscar o usuário pelo e-mail
    let user_result = query_as::<_, Usuario>("SELECT id, nome, email, senha_hash FROM usuarios WHERE email = $1")
        .bind(&login_request.email)
        .fetch_optional(&data.db_pool)
        .await;

    let user = match user_result {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Credenciais inválidas.".to_string(),
                body: None,
            });
        },
        Err(e) => {
            eprintln!("Erro ao buscar usuário para login: {:?}", e);
            return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro interno ao processar login.".to_string(),
                body: None,
            });
        }
    };

    // 2. Verificar a senha
    let password_matches = match verify(&login_request.senha, &user.senha_hash) {
        Ok(matches) => matches,
        Err(e) => {
            eprintln!("Erro ao verificar senha: {:?}", e);
            return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro interno ao verificar senha.".to_string(),
                body: None,
            });
        }
    };

    if !password_matches {
        return HttpResponse::Unauthorized().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: "Credenciais inválidas.".to_string(),
            body: None,
        });
    }

    // 3. Gerar token de autenticação (PLACEHOLDER por enquanto)
    // Em uma aplicação real, você geraria um JWT aqui.
    let auth_token = format!("mock_token_for_user_{}", user.id);

    // 4. Retornar resposta de sucesso
    HttpResponse::Ok().json(AuthResponse {
        status: "success".to_string(),
        message: "Login bem-sucedido!".to_string(),
        user_id: user.id,
        user_name: user.nome,
        user_email: user.email,
        token: auth_token,
    })
}
