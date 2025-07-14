// src/main.rs

use actix_web::{web, App, HttpServer};
use sqlx::{Pool, Postgres};
use std::sync::RwLock;


// Importa o módulo 'produtos' que contém as rotas e structs relacionadas a produtos.
// O Rust encontrará o arquivo `src/produtos/mod.rs` e, a partir dele, os submódulos.
mod produtos;
mod vendas;
mod categorias;

// Estado compartilhado que contém a conexão com o banco de dados.
// Esta struct permanece aqui, pois o pool de conexão é global para a aplicação
// e é acessado por diferentes módulos.
pub struct AppState {
    pub db_pool: Pool<Postgres>,
}

// Função principal da aplicação Actix Web.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // URL de conexão com o banco de dados PostgreSQL.
    // Certifique-se de que o tipo da coluna 'preco' no seu banco de dados PostgreSQL seja NUMERIC ou DECIMAL
    // para garantir a compatibilidade com bigdecimal::BigDecimal.
    // let database_url = "postgres://user:passsword@localhost:port/database";
    let database_url = "postgres://emanuel:Emanuel12%23@localhost:5432/bellavibe";

    // Conecta ao banco de dados PostgreSQL usando um pool de conexões.
    // O .expect() fará com que o programa entre em pânico se a conexão falhar.
    let db_pool = Pool::<Postgres>::connect(&database_url).await
        .expect("Falha ao conectar ao banco PostgreSQL");

    // Cria um estado compartilhado da aplicação com o pool de conexões.
    // web::Data é usado para compartilhar dados imutáveis entre as rotas.
    let app_state = web::Data::new(AppState { db_pool });

    // Cria e compartilha o estado do carrinho de compras em memória.
    // RwLock permite múltiplos leitores ou um único escritor.
    let carrinho_state = web::Data::new(RwLock::new(produtos::produtos_structs::Carrinho::default()));

    println!("Iniciando API BellaVibe na porta 8080...");

    // Configura e inicia o servidor HTTP.
    HttpServer::new(move || {
        App::new()
            // Adiciona o estado compartilhado à aplicação.
            // .clone() é necessário porque a closure é movida
            // e pode ser executada várias vezes.
            .app_data(app_state.clone())            
            .app_data(carrinho_state.clone())


            // Módulo de Produtos            
            .service(produtos::produtos_router::buscar_produtos)
            .service(produtos::produtos_router::cadastrar_produto)            
            .service(produtos::produtos_router::adicionar_item_sacola) 
            .service(produtos::produtos_router::ver_sacola)
                        
            //Módulo de Vendas            
            .service(vendas::vendas_router::realizar_venda)

            // Módulo de Categorias (Novas Rotas)
            .service(categorias::categoria_router::cadastrar_categoria)
            .service(categorias::categoria_router::buscar_categorias)
            .service(categorias::categoria_router::buscar_categoria_por_id)
            .service(categorias::categoria_router::atualizar_categoria)
            .service(categorias::categoria_router::deletar_categoria)
    })
    // Vincula o servidor ao endereço IP e porta. O '?' propaga erros.
    .bind("127.0.0.1:8080")?
    // Inicia o servidor. 
    .run()
    // Aguarda a finalização do servidor.                   
    .await                   
}