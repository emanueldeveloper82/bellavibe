// src/usuarios/auth_middleware.rs

use actix_web::{
    dev::Payload,
    error::ErrorUnauthorized,
    FromRequest, HttpRequest, web
};

use futures::future::{ready, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

// Importa as Claims do módulo de structs de usuário
use super::usuario_structs::Claims;
// Importa o AppState do módulo raiz (main.rs)
use crate::AppState;

/// Struct que representa o usuário autenticado, contendo as claims do JWT.
/// Será extraída das requisições protegidas.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i32,
    pub user_name: String,
    pub user_email: String,    
}

/// Extrator de autenticação para Actix Web.
/// Este extrator tenta validar um token JWT presente no cabeçalho Authorization.
impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Acessa o AppState para obter a chave secreta JWT
        let app_state = req.app_data::<web::Data<AppState>>();

        let jwt_secret = match app_state {
            Some(state) => state.jwt_secret.clone(),
            None => {
                eprintln!("Erro: AppState ou jwt_secret não disponível no extrator.");
                return ready(Err(ErrorUnauthorized("Erro de configuração do servidor.")));
            }
        };

        // Tenta obter o cabeçalho "Authorization"
        let auth_header = req.headers().get("Authorization");

        let token = match auth_header {
            Some(header_value) => {
                let header_str = match header_value.to_str() {
                    Ok(s) => s,
                    Err(_) => return ready(Err(ErrorUnauthorized("Token de autenticação inválido."))),
                };

                // Verifica se o cabeçalho começa com "Bearer "
                if header_str.starts_with("Bearer ") {
                    header_str.trim_start_matches("Bearer ").to_string()
                } else {
                    return ready(Err(ErrorUnauthorized("Formato de token inválido. Esperado 'Bearer <token>'.")));
                }
            },
            None => {
                return ready(Err(ErrorUnauthorized("Token de autenticação ausente.")));
            }
        };

        // Configuração de validação do JWT
        let validation = Validation::new(Algorithm::HS256);
        // Você pode adicionar mais validações aqui, como 'iss' (issuer) ou 'aud' (audience)
        // validation.validate_exp = true; // Já é true por padrão
        // validation.leeway = 60; // Permite uma pequena margem de erro no tempo de expiração (60 segundos)

        // Decodifica e valida o token
        let token_data = match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &validation,
        ) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Erro ao decodificar/validar JWT: {:?}", e);
                let error_message = match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token expirado.",
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => "Assinatura do token inválida.",
                    jsonwebtoken::errors::ErrorKind::InvalidToken => "Token malformado.",
                    _ => "Token de autenticação inválido.",
                };
                return ready(Err(ErrorUnauthorized(error_message)));
            }
        };

        // Se a validação for bem-sucedida, cria a instância de AuthenticatedUser
        let authenticated_user = AuthenticatedUser {
            user_id: token_data.claims.sub,
            user_name: token_data.claims.name,
            user_email: token_data.claims.email,
        };
        

        ready(Ok(authenticated_user))
    }
}
