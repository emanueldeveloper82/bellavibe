// src/main.rs

use actix_web::{web, App, HttpServer};
use sqlx::{Pool, Postgres};
use std::sync::RwLock;


// Importa os módulos
//
// Importa o módulo 'produtos' que contém as rotas e structs relacionadas a produtos.
// O Rust encontrará o arquivo `src/produtos/mod.rs` e, a partir dele, os submódulos.
mod produtos;   // Módulo de produtos
mod vendas;     // Módulo de vendas
mod categorias; // Módulo de categorias
mod shared;     // Módulo shared
mod usuarios;   // Módulo de usuários

// Estado compartilhado que contém a conexão com o banco de dados e a chave secreta JWT.
pub struct AppState {
    pub db_pool: Pool<Postgres>,
    pub jwt_secret: String, //Chave secreta para JWT
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

    // Define a chave secreta JWT (em produção, viria de variáveis de ambiente)
    //let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "(5ax<hF#<fT_pG>2poL1>XuL)345[sxY".into()); 
    let jwt_secret = "minha_chave_secreta_para_testes_123".to_string();

    // Cria um estado compartilhado da aplicação com o pool de conexões.
    // web::Data é usado para compartilhar dados imutáveis entre as rotas.
    let app_state = web::Data::new(AppState { db_pool, jwt_secret });

    // Cria e compartilha o estado do carrinho de compras em memória.
    // RwLock permite múltiplos leitores ou um único escritor.
    let carrinho_state = web::Data::new(RwLock::new(vendas::vendas_structs::Carrinho::default()));

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
            .service(produtos::produtos_router::buscar_produto_por_id)
            .service(produtos::produtos_router::cadastrar_produto)
            .service(produtos::produtos_router::atualizar_produto)
            .service(produtos::produtos_router::deletar_produto)
                        
            //Módulo de Vendas            
            .service(vendas::vendas_router::realizar_venda)
            .service(vendas::vendas_router::adicionar_item_sacola)
            .service(vendas::vendas_router::ver_sacola)

            // Módulo de Categorias (Rotas de Sessões)
            .service(categorias::categoria_router::cadastrar_sessao)
            .service(categorias::categoria_router::buscar_sessoes)
            .service(categorias::categoria_router::buscar_sessao_por_id)
            .service(categorias::categoria_router::atualizar_sessao)    
            .service(categorias::categoria_router::deletar_sessao)      

            // Módulo de Categorias (Rotas de Categorias Filhas/Genéricas)
            .service(categorias::categoria_router::cadastrar_categoria)
            .service(categorias::categoria_router::buscar_categorias_por_sessao)
            .service(categorias::categoria_router::buscar_categoria_por_id)
            .service(categorias::categoria_router::atualizar_categoria)
            .service(categorias::categoria_router::deletar_categoria)

            // Módulo de Usuários (Novas Rotas)
            .service(usuarios::usuario_router::cadastrar_usuario)
            .service(usuarios::usuario_router::login_usuario)
    })
    // Vincula o servidor ao endereço IP e porta. O '?' propaga erros.
    .bind("127.0.0.1:8080")?
    // Inicia o servidor. 
    .run()
    // Aguarda a finalização do servidor.                   
    .await                   
}