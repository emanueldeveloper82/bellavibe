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

/// Estrutura para o payload do JWT (Claims).
/// Contém informações sobre o usuário e a expiração do token.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32, // Subject (ID do usuário)
    pub name: String, // Nome do usuário
    pub email: String, // Email do usuário
    pub exp: i64, // Expiration Time (timestamp Unix)
}

/// Estrutura para a resposta de sucesso do login.
/// Agora inclui o token JWT real.
#[derive(Serialize)]
pub struct AuthResponse {
    pub status: String,
    pub message: String,
    pub user_id: i32,
    pub user_name: String,
    pub user_email: String,
    pub token: String, 
}
