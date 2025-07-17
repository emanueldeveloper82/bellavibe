// src/shared/shared_structs.rs

use serde::Serialize;

/// Estrutura genérica para padronizar as respostas da API.
/// 'T' é o tipo do corpo da resposta, que pode ser opcional.
#[derive(Serialize)]
pub struct GenericResponse<T> {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")] // Não serializa 'body' se for None
    pub body: Option<T>,
}
