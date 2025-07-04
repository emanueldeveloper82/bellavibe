// src/produtos/produtos_router.rs

use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::{query_as, Row};
use serde_json;

// Importa as structs definidas no módulo `produtos_structs` dentro da mesma pasta `produtos`
use super::produtos_structs::{NovoProduto, Produto, ProdutoResponse};

// Importa o AppState do módulo raiz (main.rs)
use crate::AppState; // Note que 'crate' se refere ao nível mais alto do seu crate (bellavibe)

/// Rota para buscar todos os produtos no banco de dados.
///
/// Esta função executa uma consulta SQL para obter todos os produtos da tabela 'produtos'.
/// Mapeia os resultados para a estrutura `Produto` e, em seguida, para `ProdutoResponse`
/// para a serialização JSON na resposta HTTP.
#[get("/produtos")]
pub async fn buscar_produtos(data: web::Data<AppState>) -> impl Responder {
    // Executa a consulta para buscar todos os produtos
    let produtos_result = query_as::<_, Produto>("SELECT id, nome, descricao, preco, estoque FROM produtos")
        .fetch_all(&data.db_pool)
        .await;

    match produtos_result {
        Ok(produtos) => {
            // Mapeia a lista de 'Produto' para uma lista de 'ProdutoResponse'
            let response: Vec<ProdutoResponse> = produtos.into_iter()
                .map(|p| ProdutoResponse {
                    id: p.id,
                    nome: p.nome,
                    descricao: p.descricao,
                    preco: p.preco,
                    estoque: p.estoque,
                })
                .collect();

            // Retorna a lista de produtos como JSON com status OK
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            // Em caso de erro, imprime o erro no console e retorna um erro 500
            eprintln!("Erro ao buscar produtos: {:?}", e);
            HttpResponse::InternalServerError().body("Erro ao buscar produtos")
        }
    }
}

/// Rota para inserir um novo produto no banco de dados.
///
/// Recebe os dados do novo produto via JSON no corpo da requisição.
/// Insere o produto na tabela 'produtos' e retorna o ID gerado.
#[post("/produtos")]
pub async fn cadastrar_produto(
    data: web::Data<AppState>,
    item: web::Json<NovoProduto>, // O corpo da requisição JSON é desserializado para NovoProduto
) -> HttpResponse {
    // Executa a query SQL para inserir um novo produto e retornar o ID gerado
    let result = sqlx::query(
        "INSERT INTO produtos (nome, descricao, preco, estoque) VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(&item.nome)      // Binda o nome do produto
    .bind(&item.descricao) // Binda a descrição do produto
    .bind(&item.preco)     // Binda o preço do produto (BigDecimal)
    .bind(item.estoque)    // Binda o estoque do produto
    .fetch_one(&data.db_pool) // Executa a query e espera por uma única linha de retorno (o ID)
    .await;

    match result {
        Ok(row) => {
            // Tenta obter o ID gerado automaticamente pelo banco de dados
            match row.try_get::<i32, &str>("id") {
                Ok(id) => {
                    // Retorna o ID do novo produto em formato JSON com status OK
                    HttpResponse::Ok().json(serde_json::json!({ "id": id }))
                },
                Err(e) => {
                    // Em caso de erro ao obter o ID, imprime o erro e retorna um erro 500
                    eprintln!("Erro ao obter id do novo produto: {:?}", e);
                    HttpResponse::InternalServerError().body("Erro ao processar resposta")
                }
            }
        }
        Err(e) => {
            // Em caso de erro na inserção do produto, imprime o erro e retorna um erro 500
            eprintln!("Erro ao inserir produto: {:?}", e);
            HttpResponse::InternalServerError().body("Erro ao inserir produto")
        }
    }
}
