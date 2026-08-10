use rskafka::client::ClientBuilder;
use rskafka::record::Record;
use rskafka::client::partition::{PartitionClient, UnknownTopicHandling, Compression, OffsetAt};
use std::sync::Arc;
use chrono::Utc;
use sqlx::SqlitePool;
use crate::domain::models::SecurityEvent;
use crate::use_cases::detection::analyze_event;

#[derive(Clone)]
pub struct KafkaProducer {
    partition_client: Arc<PartitionClient>,
}

impl KafkaProducer {
    pub async fn new(broker: &str, topic: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Connexion au broker Kafka
        let client = ClientBuilder::new(vec![broker.to_string()]).build().await?;
        // Sélection de la partition 0 du topic
        let partition_client = client.partition_client(topic, 0, UnknownTopicHandling::Error).await?;
        
        Ok(Self {
            partition_client: Arc::new(partition_client),
        })
    }

    pub async fn send_event(&self, payload: &str) -> Result<(), Box<dyn std::error::Error>> {
        let record = Record {
            key: None,
            value: Some(payload.as_bytes().to_vec()),
            headers: Default::default(),
            timestamp: Utc::now(),
        };

        // Envoi du message dans Kafka
        self.partition_client
            .produce(vec![record], Compression::NoCompression)
            .await?;
            
        Ok(())
    }

    pub async fn start_consumer(
        broker: &str, 
        topic: &str, 
        pool: SqlitePool
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = ClientBuilder::new(vec![broker.to_string()]).build().await?;
        let partition_client = client.partition_client(topic, 0, UnknownTopicHandling::Error).await?;

        // On commence à lire à partir du dernier offset (derniers messages)
        let mut offset = match partition_client.get_offset(OffsetAt::Latest).await {
            Ok(off) => off,
            Err(_) => 0,
        };

        println!("🎧 [WORKER KAFKA] Écoute en continu du topic '{}' démarrée...", topic);

        loop {
            match partition_client.fetch_records(offset, 1..1_000_000, 1_000).await {
                Ok((records, _high_watermark)) => {
                    for record_and_offset in records {
                        offset = record_and_offset.offset + 1;
                        let record = record_and_offset.record;
                        if let Some(value) = record.value {
                            if let Ok(event_str) = String::from_utf8(value) {
                                if let Ok(event) = serde_json::from_str::<SecurityEvent>(&event_str) {
                                    // Analyse de l'événement via notre moteur de règles
                                    analyze_event(&event, &pool).await;
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }
        }
    }
}
