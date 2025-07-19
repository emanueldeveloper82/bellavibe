// src/usuarios/usuario_structs.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Estrutura que representa um usuário no banco de dados.
/// A senha será armazenada como um hash.
#[derive(Serialize, FromRow)]
pub struct Usuario {
    pub id: i32,
    pub nome: String,
    pub email: String,
    pub senha_hash: String, // Armazenará o hash da senha
}

/// Estrutura para receber dados de um novo usuário na requisição de cadastro.
#[derive(Deserialize)]
pub struct NovoUsuario {
    pub nome: String,
    pub email: String,
    pub senha: String, // Senha em texto claro (será hashed antes de salvar)
}

/// Estrutura para receber dados de login do usuário.
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub senha: String, // Senha em texto claro
}

/// Estrutura para a resposta de sucesso do login.
/// Por enquanto, um placeholder para o token.
#[derive(Serialize)]
pub struct AuthResponse {
    pub status: String,
    pub message: String,
    pub user_id: i32,
    pub user_name: String,
    pub user_email: String,
    pub token: String, // Token de autenticação (JWT ou similar)
}
