use sqlx::SqlitePool;
use uuid::Uuid;
use crate::domain::models::SecurityEvent;

pub async fn analyze_event(event: &SecurityEvent, pool: &SqlitePool) {
    // Règle 1 : Détection d'échec de connexion avec sévérité Haute ou CRITICAL
    if event.event_type == "FAILED_LOGIN" && (event.severity == "HIGH" || event.severity == "CRITICAL") {
        let alert_id = Uuid::new_v4().to_string();
        let title = format!("Tentative de connexion suspecte sur {}", event.source);
        let description = format!("L'agent {} a rapporté une tentative échouée.", event.agent_id);

        println!("⚠️ [ALERTE DÉTECTÉE] {} - Agent: {}", title, event.agent_id);

        // Insertion de l'alerte dans SQLite
        let result = sqlx::query(
            "INSERT INTO alerts (id, title, description, severity, status, created_at) VALUES (?, ?, ?, ?, 'OPEN', datetime('now'))"
        )
        .bind(&alert_id)
        .bind(&title)
        .bind(&description)
        .bind(&event.severity)
        .execute(pool)
        .await;

        match result {
            Ok(_) => println!("💾 [DATABASE] Alerte enregistrée avec succès (ID: {})", alert_id),
            Err(e) => eprintln!("❌ [DATABASE] Erreur d'enregistrement d'alerte: {:?}", e),
        }
    }
}
