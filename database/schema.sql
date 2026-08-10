-- Activation des clés étrangères dans SQLite
PRAGMA foreign_keys = ON;

-- Table des rôles
CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY, -- Stockage des UUID en texte
    name TEXT UNIQUE NOT NULL
);

-- Table des utilisateurs
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role_id TEXT,
    is_active INTEGER DEFAULT 1, -- 1 pour TRUE, 0 pour FALSE
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (role_id) REFERENCES roles (id)
);

-- Table des machines supervisées
CREATE TABLE IF NOT EXISTS machines (
    id TEXT PRIMARY KEY,
    hostname TEXT NOT NULL,
    os TEXT,
    ip_address TEXT,
    status TEXT DEFAULT 'unknown',
    last_seen DATETIME
);

-- Table des agents de collecte
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    version TEXT,
    api_key TEXT UNIQUE,
    machine_id TEXT,
    last_heartbeat DATETIME,
    FOREIGN KEY (machine_id) REFERENCES machines (id)
);

-- Table des événements (JSON stocké en TEXT, SQLite possède des fonctions JSON natives)
CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT,
    event_type TEXT,
    source TEXT,
    raw_data TEXT, -- Objet JSON sérialisé en texte
    received_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (agent_id) REFERENCES agents (id)
);

-- Table des règles de détection
CREATE TABLE IF NOT EXISTS rules (
    id TEXT PRIMARY KEY,
    name TEXT,
    description TEXT,
    condition TEXT, -- Condition stockée en texte JSON
    severity TEXT,
    is_active INTEGER DEFAULT 1
);

-- Table des alertes
CREATE TABLE IF NOT EXISTS alerts (
    id TEXT PRIMARY KEY,
    rule_id TEXT,
    event_id INTEGER,
    title TEXT,
    description TEXT,
    severity TEXT,
    status TEXT DEFAULT 'OPEN',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (rule_id) REFERENCES rules (id),
    FOREIGN KEY (event_id) REFERENCES events (id)
);

-- Table des incidents
CREATE TABLE IF NOT EXISTS incidents (
    id TEXT PRIMARY KEY,
    title TEXT,
    status TEXT DEFAULT 'open',
    assigned_to TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (assigned_to) REFERENCES users (id)
);

-- Table d'audit
CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT,
    action TEXT,
    target TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id)
);