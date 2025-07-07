// src/main.rs

use actix_web::{web, App, HttpServer};
use sqlx::{Pool, Postgres};

// Importa o módulo 'produtos' que contém as rotas e structs relacionadas a produtos.
// O Rust encontrará o arquivo `src/produtos/mod.rs` e, a partir dele, os submódulos.
mod produtos;

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

    println!("Iniciando API BellaVibe na porta 8080...");

    // Configura e inicia o servidor HTTP.
    HttpServer::new(move || {
        App::new()
            // Adiciona o estado compartilhado à aplicação.
            // .clone() é necessário porque a closure é movida
            // e pode ser executada várias vezes.
            .app_data(app_state.clone())

            // Registra as rotas importadas do submódulo `produtos_router`.
            // Note o caminho completo: `produtos::produtos_router::`
            .service(produtos::produtos_router::buscar_produtos)
            .service(produtos::produtos_router::cadastrar_produto)
            .service(produtos::produtos_router::realizar_venda)
    })
    // Vincula o servidor ao endereço IP e porta. O '?' propaga erros.
    .bind("127.0.0.1:8080")?
    // Inicia o servidor. 
    .run()
    // Aguarda a finalização do servidor.                   
    .await                   
}