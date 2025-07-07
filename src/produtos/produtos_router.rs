// src/produtos/produtos_router.rs

use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::{query_as, Row};
use serde_json;
use bigdecimal::BigDecimal;
use std::sync::RwLock; // Para a sacola em memória 


// Importa as structs definidas no módulo `produtos_structs` dentro da mesma pasta `produtos`
// Certifique-se de que todas as structs necessárias estão listadas aqui
use super::produtos_structs::{
    GenericResponse,
    NovoProduto,
    Produto,
    ProdutoResponse,
    ItemVenda,
    VendaResponse,    
    Carrinho       
};

// Importa o AppState do módulo raiz (main.rs)
use crate::AppState; // Note que 'crate' se refere ao nível mais alto do seu crate (bellavibe)

/// Rota para buscar todos os produtos no banco de dados.
///
/// Esta função executa uma consulta SQL para obter todos os produtos da tabela 'produtos'.
/// Mapeia os resultados para a estrutura `Produto` e, em seguida, para `ProdutoResponse`
/// para a serialização JSON na resposta HTTP.
/// Retorna uma GenericResponse com a lista de produtos.
#[get("/produtos")]
pub async fn buscar_produtos(data: web::Data<AppState>) -> impl Responder {
    let produtos_result = query_as::<_, Produto>("SELECT id, nome, descricao, preco, estoque FROM produtos")
        .fetch_all(&data.db_pool)
        .await;

    match produtos_result {
        Ok(produtos) => {
            let response_body: Vec<ProdutoResponse> = produtos.into_iter()
                .map(|p| ProdutoResponse {
                    id: p.id,
                    nome: p.nome,
                    descricao: p.descricao,
                    preco: p.preco,
                    estoque: p.estoque,
                })
                .collect();
            
            // Retorna uma GenericResponse de sucesso com o corpo contendo a lista de produtos
            HttpResponse::Ok().json(GenericResponse {
                status: "success".to_string(),
                message: "Produtos listados com sucesso!".to_string(),
                body: Some(response_body),
            })
        },
        Err(e) => {
            eprintln!("Erro ao buscar produtos: {:?}", e);
            // Retorna uma GenericResponse de erro
            HttpResponse::InternalServerError().json(GenericResponse::<()>{ // Usamos <()> para um corpo vazio
                status: "error".to_string(),
                message: "Erro ao buscar produtos".to_string(),
                body: None,
            })
        }
    }
}

/// Rota para inserir um novo produto no banco de dados.
///
/// Recebe os dados do novo produto via JSON no corpo da requisição.
/// Insere o produto na tabela 'produtos' e retorna o ID gerado.
/// Retorna uma GenericResponse com o ID do produto criado.
#[post("/produtos")]
pub async fn cadastrar_produto(
    data: web::Data<AppState>,
    item: web::Json<NovoProduto>,
) -> HttpResponse {
    let result = sqlx::query(
        "INSERT INTO produtos (nome, descricao, preco, estoque) VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(&item.nome)
    .bind(&item.descricao)
    .bind(&item.preco)
    .bind(item.estoque)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(row) => {
            match row.try_get::<i32, &str>("id") {
                Ok(id) => {
                    // Retorna uma GenericResponse de sucesso com o ID
                    HttpResponse::Ok().json(GenericResponse {
                        status: "success".to_string(),
                        message: format!("Produto cadastrado com sucesso! ID: {}", id),
                        body: Some(serde_json::json!({ "id": id })), // Usando serde_json::Value para o body
                    })
                },
                Err(e) => {
                    eprintln!("Erro ao obter id do novo produto: {:?}", e);
                    // Retorna uma GenericResponse de erro
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
            // Retorna uma GenericResponse de erro
            HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao inserir produto".to_string(),
                body: None,
            })
        }
    }
}



/// Rota para adicionar um item à sacola de compras.
/// Recebe um ItemVenda no corpo da requisição.
#[post("/sacola/adicionar")]
pub async fn adicionar_item_sacola(
    carrinho_data: web::Data<RwLock<Carrinho>>, // Acesso ao estado da sacola
    item_venda: web::Json<ItemVenda>,
    data: web::Data<AppState>, // Necessário para verificar o produto no DB
) -> HttpResponse {
    // Verifica se o produto existe no banco de dados
    let produto_exists = sqlx::query_as::<_, Produto>(
        "SELECT id, nome, descricao, preco, estoque FROM produtos WHERE id = $1"
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
    
    
    // Retorna uma GenericResponse de sucesso com o corpo contendo os itens da sacola
    HttpResponse::Ok().json(GenericResponse {
        status: "success".to_string(),
        message: "Conteúdo da sacola".to_string(),
        body: Some(carrinho.itens.clone()), // Clona os itens para a resposta
    })
}

/// Rota para realizar uma venda de produtos, consumindo itens da sacola.
///
/// Realiza as seguintes operações dentro de uma transação:
/// 1. Verifica a disponibilidade de estoque para cada item na sacola.
/// 2. Calcula o valor total da compra.
/// 3. Decrementa o estoque dos produtos vendidos.
/// 4. Limpa a sacola após a venda.
/// Retorna o valor total da compra em caso de sucesso ou um erro em caso de falha (ex: estoque insuficiente).
#[post("/venda")]
pub async fn realizar_venda(
    data: web::Data<AppState>,
    carrinho_data: web::Data<RwLock<Carrinho>>, // Acesso ao estado da sacola
) -> HttpResponse {
    // Pega os itens da sacola e limpa-a. Isso é feito dentro de um bloco para liberar o lock de escrita rapidamente.
    let itens_venda = {
        let mut carrinho = carrinho_data.write().unwrap();
        if carrinho.itens.is_empty() {
            // Retorna uma GenericResponse de erro, explicitando o tipo genérico como `()`
            return HttpResponse::BadRequest().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "A sacola está vazia. Adicione itens antes de realizar a venda.".to_string(),
                body: None,
            });
        }
        std::mem::take(&mut carrinho.itens) // Pega os itens e deixa o vetor vazio
    };

    // Inicia uma transação no banco de dados para garantir atomicidade
    let mut transaction = match data.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Erro ao iniciar transação: {:?}", e);
            // Retorna uma GenericResponse de erro, explicitando o tipo genérico como `()`
            return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro interno ao processar venda".to_string(),
                body: None,
            });
        }
    };

    let mut total_compra = BigDecimal::from(0); // Inicializa o total da compra com 0

    // Itera sobre cada item na sacola
    for item in itens_venda.iter() {
        // 1. Busca o produto no banco de dados para verificar estoque e preço
        // FOR UPDATE bloqueia a linha para evitar race conditions em ambientes multi-usuário
        let produto_result = sqlx::query_as::<_, Produto>(
            "SELECT id, nome, descricao, preco, estoque FROM produtos WHERE id = $1 FOR UPDATE"
        )
        .bind(item.produto_id)
        .fetch_optional(&mut *transaction) // Usa a transação para a consulta
        .await;

        let produto = match produto_result {
            Ok(Some(p)) => p,
            Ok(None) => {
                eprintln!("Produto com ID {} não encontrado durante a venda.", item.produto_id);
                let _ = transaction.rollback().await;
                // Retorna uma GenericResponse de erro, explicitando o tipo genérico como `()`
                return HttpResponse::BadRequest().json(GenericResponse::<()>{
                    status: "error".to_string(),
                    message: format!("Produto com ID {} não encontrado para venda.", item.produto_id),
                    body: None,
                });
            },
            Err(e) => {
                eprintln!("Erro ao buscar produto {}: {:?}", item.produto_id, e);
                let _ = transaction.rollback().await;
                // Retorna uma GenericResponse de erro, explicitando o tipo genérico como `()`
                return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                    status: "error".to_string(),
                    message: "Erro ao buscar produto para venda".to_string(),
                    body: None,
                });
            }
        };

        // 2. Verifica se há estoque suficiente
        if produto.estoque < item.quantidade {
            eprintln!("Estoque insuficiente para o produto {}. Disponível: {}, Solicitado: {}",
                      produto.nome, produto.estoque, item.quantidade);
            let _ = transaction.rollback().await;
            // Retorna uma GenericResponse de erro, explicitando o tipo genérico como `()`
            return HttpResponse::BadRequest().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: format!("Estoque insuficiente para o produto {}.", produto.nome),
                body: None,
            });
        }

        // Calcula o subtotal para o item e adiciona ao total da compra
        let quantidade_bigdecimal = BigDecimal::from(item.quantidade);
        let subtotal = &produto.preco * &quantidade_bigdecimal;
        total_compra += subtotal;

        // 3. Decrementa o estoque do produto
        let novo_estoque = produto.estoque - item.quantidade;
        let update_result = sqlx::query(
            "UPDATE produtos SET estoque = $1 WHERE id = $2"
        )
        .bind(novo_estoque)
        .bind(item.produto_id)
        .execute(&mut *transaction)
        .await;

        if let Err(e) = update_result {
            eprintln!("Erro ao atualizar estoque do produto {}: {:?}", produto.nome, e);
            let _ = transaction.rollback().await;
            // Retorna uma GenericResponse de erro, explicitando o tipo genérico como `()`
            return HttpResponse::InternalServerError().json(GenericResponse::<()>{
                status: "error".to_string(),
                message: "Erro ao atualizar estoque durante a venda".to_string(),
                body: None,
            });
        }
    }

    // Se todas as operações foram bem-sucedidas, comita a transação
    if let Err(e) = transaction.commit().await {
        eprintln!("Erro ao comitar transação: {:?}", e);
        // Retorna uma GenericResponse de erro, explicitando o tipo genérico como `()`
        return HttpResponse::InternalServerError().json(GenericResponse::<()>{
            status: "error".to_string(),
            message: "Erro interno ao finalizar venda".to_string(),
            body: None,
        });
    }

    // Retorna a resposta de sucesso com o total da compra
    HttpResponse::Ok().json(GenericResponse {
        status: "success".to_string(),
        message: "Venda realizada com sucesso!".to_string(),
        body: Some(VendaResponse {
            total_compra,
            mensagem: "Venda processada e sacola limpa.".to_string(),
        }),
    })
}

