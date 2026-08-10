#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub role_id: Option<String>,
    pub is_active: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub agent_id: String,
    pub event_type: String, // ex: "FAILED_LOGIN", "PROCESS_CREATION", "NETWORK_CONN"
    pub source: String,     // ex: "192.168.1.50" ou "AuthService"
    pub severity: String,   // ex: "LOW", "MEDIUM", "HIGH", "CRITICAL"
    pub raw_data: serde_json::Value, // Contenu brut du log au format JSON
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Machine {
    pub id: String,
    pub hostname: String,
    pub os: Option<String>,
    pub ip_address: Option<String>,
    pub status: Option<String>,
    pub last_seen: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Agent {
    pub id: String,
    pub version: Option<String>,
    pub api_key: Option<String>,
    pub machine_id: Option<String>,
    pub last_heartbeat: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: Option<i64>,
    pub agent_id: Option<String>,
    pub event_type: Option<String>,
    pub source: Option<String>,
    pub raw_data: Option<String>,
    pub received_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Rule {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub condition: Option<String>,
    pub severity: Option<String>,
    pub is_active: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alert {
    pub id: String,
    pub rule_id: Option<String>,
    pub event_id: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Incident {
    pub id: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub assigned_to: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Option<i64>,
    pub user_id: Option<String>,
    pub action: Option<String>,
    pub target: Option<String>,
    pub created_at: Option<String>,
}