// src/produtos/produtos_router.rs

use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::{query_as, Row};
use serde_json;
use bigdecimal::BigDecimal; 

// Importa as structs definidas no módulo `produtos_structs` dentro da mesma pasta `produtos`
// Certifique-se de que todas as structs necessárias estão listadas aqui
use super::produtos_structs::{
    NovoProduto,
    Produto,
    ProdutoResponse,    
    VendaRequest,   
    VendaResponse   
};

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


/// Rota para realizar uma venda de produtos.
///
/// Recebe uma lista de itens de venda (produto_id e quantidade) no corpo da requisição.
/// Realiza as seguintes operações dentro de uma transação:
/// 1. Verifica a disponibilidade de estoque para cada item.
/// 2. Calcula o valor total da compra.
/// 3. Decrementa o estoque dos produtos vendidos.
/// Retorna o valor total da compra em caso de sucesso ou um erro em caso de falha (ex: estoque insuficiente).
#[post("/venda")]
pub async fn realizar_venda(
    data: web::Data<AppState>,
    venda_req: web::Json<VendaRequest>, // Requisição de venda contendo os itens
) -> HttpResponse {
    // Inicia uma transação no banco de dados para garantir atomicidade
    let mut transaction = match data.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Erro ao iniciar transação: {:?}", e);
            return HttpResponse::InternalServerError().body("Erro interno ao processar venda");
        }
    };

    // Inicializa o total da compra com 0
    let mut total_compra = BigDecimal::from(0); 

    // Itera sobre cada item na requisição de venda
    for item in venda_req.itens.iter() {
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
                // Produto não encontrado
                eprintln!("Produto com ID {} não encontrado.", item.produto_id);
                // Rollback da transação em caso de erro
                let _ = transaction.rollback().await;
                return HttpResponse::BadRequest().body(format!("Produto com ID {} não encontrado.", item.produto_id));
            },
            Err(e) => {
                // Erro ao buscar produto
                eprintln!("Erro ao buscar produto {}: {:?}", item.produto_id, e);
                let _ = transaction.rollback().await;
                return HttpResponse::InternalServerError().body("Erro ao buscar produto");
            }
        };

        // 2. Verifica se há estoque suficiente
        if produto.estoque < item.quantidade {
            eprintln!("Estoque insuficiente para o produto {}. Disponível: {}, Solicitado: {}",
                      produto.nome, produto.estoque, item.quantidade);
            // Rollback da transação em caso de estoque insuficiente
            let _ = transaction.rollback().await;
            return HttpResponse::BadRequest().body(format!("Estoque insuficiente para o produto {}.", produto.nome));
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
        .execute(&mut *transaction) // Usa a transação para a atualização
        .await;

        if let Err(e) = update_result {
            eprintln!("Erro ao atualizar estoque do produto {}: {:?}", produto.nome, e);
            let _ = transaction.rollback().await;
            return HttpResponse::InternalServerError().body("Erro ao atualizar estoque");
        }
    }

    // Se todas as operações foram bem-sucedidas, comita a transação
    if let Err(e) = transaction.commit().await {
        eprintln!("Erro ao comitar transação: {:?}", e);
        return HttpResponse::InternalServerError().body("Erro interno ao finalizar venda");
    }

    // Retorna a resposta de sucesso com o total da compra
    HttpResponse::Ok().json(VendaResponse {
        total_compra,
        mensagem: "Venda realizada com sucesso!".to_string(),
    })
}
