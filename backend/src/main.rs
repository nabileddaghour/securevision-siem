mod domain;
mod infrastructure;
mod use_cases;

use sqlx::sqlite::SqlitePoolOptions;
use std::env;
use dotenvy::dotenv;
use infrastructure::kafka::KafkaProducer;
use infrastructure::redis::RedisCache;
use infrastructure::web::AppState;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL introuvable.");
    let kafka_broker = env::var("KAFKA_BROKER").unwrap_or_else(|_| "localhost:9092".to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("Échec de la connexion à SQLite.");
        
    println!("✅ Connexion SQLite réussie !");

    // Tentative d'initialisation de Redis
    let redis = match RedisCache::new(&redis_url) {
        Ok(r) => {
            println!("✅ Configuration du client Redis enregistrée ({}).", redis_url);
            Some(r)
        }
        Err(_) => {
            println!("⚠️ Client Redis non disponible.");
            None
        }
    };

    // Tentative d'initialisation de Kafka avec un timeout rapide de 2 secondes
    let kafka_res = timeout(Duration::from_secs(2), KafkaProducer::new(&kafka_broker, "siem-events")).await;
    let kafka = match kafka_res {
        Ok(Ok(producer)) => {
            println!("✅ Connexion Producer Kafka réussie !");
            Some(producer)
        }
        _ => {
            println!("⚠️ Kafka non joignable - Mode dégradé activé");
            None
        }
    };

    // Lancement du Consumer Kafka uniquement si Kafka est joignable
    if kafka.is_some() {
        let pool_clone = pool.clone();
        let broker_clone = kafka_broker.clone();
        tokio::spawn(async move {
            if let Err(e) = KafkaProducer::start_consumer(&broker_clone, "siem-events", pool_clone).await {
                eprintln!("❌ Erreur sur le Consumer Kafka: {:?}", e);
            }
        });
    }

    let state = AppState { pool, kafka, redis };
    let app = infrastructure::web::create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Serveur SIEM & Moteur de Détection démarrés sur http://localhost:3000");
    
    axum::serve(listener, app).await.unwrap();
}