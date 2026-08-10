use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    routing::{get, post},
    Json, Router,
};
use sqlx::SqlitePool;
use uuid::Uuid;
use crate::domain::models::{Alert, AuthResponse, Claims, LoginRequest, RegisterRequest, SecurityEvent, User};
use crate::infrastructure::kafka::KafkaProducer;
use crate::infrastructure::redis::RedisCache;
use crate::use_cases::auth::{create_jwt, decode_jwt, hash_password, verify_password};

// Structure d'état partagée entre toutes les routes Axum
#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub kafka: Option<KafkaProducer>, // Optionnel pour ne pas bloquer si Kafka est arrêté en dev
    pub redis: Option<RedisCache>,   // Cache Redis optionnel pour les sessions/stats
}

// Extracteur Axum pour valider le Token JWT (Auth Guard)
#[allow(dead_code)]
pub struct AuthUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|val| val.to_str().ok());

        let token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => &header[7..],
            _ => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({ "error": "Token d'authentification manquant ou invalide" })),
                ));
            }
        };

        match decode_jwt(token) {
            Ok(claims) => Ok(AuthUser(claims)),
            Err(_) => Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Token invalide ou expiré" })),
            )),
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(|| async { "L'API Gateway du SIEM est en ligne !" }))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/users", get(get_users))
        .route("/api/alerts", get(get_alerts))
        .route("/api/events", post(ingest_event))
        .with_state(state)
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let user_id = Uuid::new_v4().to_string();
    let hashed_pw = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(_) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Erreur lors du hachage du mot de passe" })),
        ),
    };
    let role = payload.role.unwrap_or_else(|| "analyst".to_string());

    // Assurer la présence du rôle dans la table roles pour satisfaire la clé étrangère
    let _ = sqlx::query("INSERT OR IGNORE INTO roles (id, name) VALUES (?, ?)")
        .bind(&role)
        .bind(&role)
        .execute(&state.pool)
        .await;

    let result = sqlx::query(
        "INSERT INTO users (id, email, password_hash, role_id, is_active) VALUES (?, ?, ?, ?, 1)"
    )
    .bind(&user_id)
    .bind(&payload.email)
    .bind(&hashed_pw)
    .bind(&role)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "status": "success",
                "user_id": user_id,
                "message": "Utilisateur créé avec succès"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("Impossible de créer l'utilisateur: {:?}", e) })),
        ),
    }
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let user = sqlx::query_as::<_, User>("SELECT id, email, password_hash, role_id, is_active FROM users WHERE email = ?")
        .bind(&payload.email)
        .fetch_optional(&state.pool)
        .await;

    let user = match user {
        Ok(Some(u)) => u,
        _ => return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Email ou mot de passe incorrect" })),
        ),
    };

    if !verify_password(&user.password_hash, &payload.password) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Email ou mot de passe incorrect" })),
        );
    }

    let role = user.role_id.unwrap_or_else(|| "analyst".to_string());
    let token = match create_jwt(&user.id, &user.email, &role) {
        Ok(t) => t,
        Err(_) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Échec de génération du token JWT" })),
        ),
    };

    (
        StatusCode::OK,
        Json(serde_json::json!(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
        })),
    )
}

async fn get_users(_auth: AuthUser, State(state): State<AppState>) -> Json<Vec<User>> {
    let users = sqlx::query_as::<_, User>("SELECT id, email, password_hash, role_id, is_active FROM users")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    Json(users)
}

async fn get_alerts(_auth: AuthUser, State(state): State<AppState>) -> Json<Vec<Alert>> {
    let alerts = sqlx::query_as::<_, Alert>(
        "SELECT id, rule_id, event_id, title, description, severity, status, created_at FROM alerts"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    Json(alerts)
}

async fn ingest_event(
    State(state): State<AppState>,
    Json(payload): Json<SecurityEvent>,
) -> (StatusCode, Json<serde_json::Value>) {
    println!("📥 [LOG REÇU] Agent: {} | Type: {} | Sévérité: {}", 
        payload.agent_id, payload.event_type, payload.severity);

    // Analyse en temps réel via le Moteur de Détection
    crate::use_cases::detection::analyze_event(&payload, &state.pool).await;

    // Sérialisation du log en JSON pour Kafka
    let payload_str = match serde_json::to_string(&payload) {
        Ok(s) => s,
        Err(_) => return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Échec de sérialisation du log" })),
        ),
    };

    // Publication dans Kafka si le client est actif
    if let Some(kafka) = &state.kafka {
        if let Err(e) = kafka.send_event(&payload_str).await {
            eprintln!("❌ Erreur d'envoi dans Kafka: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Erreur d'ingestion Kafka" })),
            );
        }
        println!("🚀 [KAFKA] Événement transmis avec succès au topic 'siem-events'");
    } else {
        println!("⚠️ [MODE DÉGRADÉ] Kafka non connecté, log ignoré pour Kafka.");
    }

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "success",
            "message": "Événement réceptionné et transmis à Kafka"
        })),
    )
}