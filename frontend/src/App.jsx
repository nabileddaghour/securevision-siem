import React, { useState, useEffect } from 'react';
import { Shield, AlertTriangle, Activity, Server, Lock, LogOut, CheckCircle, RefreshCw } from 'lucide-react';

export default function App() {
  const [token, setToken] = useState(localStorage.getItem('jwt_token') || '');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [alerts, setAlerts] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [isLoginOpen, setIsLoginOpen] = useState(!token);

  const fetchAlerts = async () => {
    if (!token) return;
    setLoading(true);
    setError('');
    try {
      const res = await fetch('/api/alerts', {
        headers: { Authorization: `Bearer ${token}` }
      });
      if (res.ok) {
        const data = await res.json();
        setAlerts(data);
      } else if (res.status === 401) {
        setError('Session expirée. Veuillez vous reconnecter.');
        handleLogout();
      }
    } catch (err) {
      setError('Erreur de connexion avec le serveur SIEM.');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (token) {
      fetchAlerts();
      const interval = setInterval(fetchAlerts, 5000);
      return () => clearInterval(interval);
    }
  }, [token]);

  const handleLogin = async (e) => {
    e.preventDefault();
    setError('');
    try {
      const res = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password })
      });
      const data = await res.json();
      if (res.ok && data.token) {
        localStorage.setItem('jwt_token', data.token);
        setToken(data.token);
        setIsLoginOpen(false);
      } else {
        setError(data.error || 'Échec de connexion');
      }
    } catch (err) {
      setError('Erreur réseau lors de la connexion');
    }
  };

  const handleLogout = () => {
    localStorage.removeItem('jwt_token');
    setToken('');
    setAlerts([]);
    setIsLoginOpen(true);
  };

  return (
    <div className="app-container">
      <nav className="navbar">
        <div className="brand">
          <Shield className="brand-icon" size={28} />
          <span>SecureVision SIEM</span>
        </div>
        <div className="nav-status">
          <div className="status-badge">
            <span className="pulse-dot"></span>
            <span>Système En Ligne</span>
          </div>
          {token ? (
            <button className="btn-login" onClick={handleLogout} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <LogOut size={16} /> Déconnexion
            </button>
          ) : (
            <button className="btn-login" onClick={() => setIsLoginOpen(true)}>
              <Lock size={16} /> Connexion
            </button>
          )}
        </div>
      </nav>

      <main className="content-grid">
        {/* Modal de Connexion JWT */}
        {isLoginOpen && (
          <div style={{
            position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.75)', backdropFilter: 'blur(8px)',
            display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 100
          }}>
            <div style={{
              background: '#111827', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '1rem',
              padding: '2.5rem', width: '100%', maxWidth: '400px', boxShadow: '0 25px 50px -12px rgba(0,0,0,0.5)'
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '1.5rem' }}>
                <Shield size={32} color="#60a5fa" />
                <h2 style={{ fontSize: '1.5rem', fontWeight: 800 }}>Authentification SIEM</h2>
              </div>
              
              {error && <div style={{ background: 'rgba(239,68,68,0.15)', color: '#f87171', padding: '0.75rem', borderRadius: '0.5rem', fontSize: '0.85rem', marginBottom: '1rem' }}>{error}</div>}

              <form onSubmit={handleLogin} style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                <div>
                  <label style={{ fontSize: '0.8rem', color: '#9ca3af', display: 'block', marginBottom: '0.35rem' }}>Adresse Email</label>
                  <input
                    type="email" required value={email} onChange={e => setEmail(e.target.value)}
                    placeholder="admin@securevision.io"
                    style={{ width: '100%', padding: '0.75rem', background: '#1f2937', border: '1px solid #374151', borderRadius: '0.5rem', color: '#fff', outline: 'none' }}
                  />
                </div>
                <div>
                  <label style={{ fontSize: '0.8rem', color: '#9ca3af', display: 'block', marginBottom: '0.35rem' }}>Mot de Passe</label>
                  <input
                    type="password" required value={password} onChange={e => setPassword(e.target.value)}
                    placeholder="••••••••"
                    style={{ width: '100%', padding: '0.75rem', background: '#1f2937', border: '1px solid #374151', borderRadius: '0.5rem', color: '#fff', outline: 'none' }}
                  />
                </div>
                <button type="submit" className="btn-login" style={{ marginTop: '0.5rem', padding: '0.85rem' }}>
                  Se Connecter (JWT)
                </button>
              </form>
            </div>
          </div>
        )}

        {/* Stats Cards */}
        <div className="stats-container">
          <div className="stat-card">
            <div>
              <div className="stat-title">Alertes Détectées</div>
              <div className="stat-value" style={{ color: '#ef4444' }}>{alerts.length}</div>
            </div>
            <div className="stat-icon" style={{ background: 'rgba(239, 68, 68, 0.15)', color: '#ef4444' }}>
              <AlertTriangle size={24} />
            </div>
          </div>

          <div className="stat-card">
            <div>
              <div className="stat-title">Agents Supervisés</div>
              <div className="stat-value" style={{ color: '#60a5fa' }}>3</div>
            </div>
            <div className="stat-icon" style={{ background: 'rgba(59, 130, 246, 0.15)', color: '#60a5fa' }}>
              <Server size={24} />
            </div>
          </div>

          <div className="stat-card">
            <div>
              <div className="stat-title">Pipeline Kafka</div>
              <div className="stat-value" style={{ color: '#10b981', fontSize: '1.25rem' }}>Actif</div>
            </div>
            <div className="stat-icon" style={{ background: 'rgba(16, 185, 129, 0.15)', color: '#10b981' }}>
              <Activity size={24} />
            </div>
          </div>
        </div>

        {/* Tableau des Alertes */}
        <div className="table-panel">
          <div className="panel-header">
            <div className="panel-title" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <AlertTriangle color="#ef4444" size={20} />
              <span>Alertes de Sécurité en Temps Réel</span>
            </div>
            <button
              onClick={fetchAlerts}
              style={{ background: 'transparent', border: '1px solid var(--panel-border)', color: '#9ca3af', padding: '0.4rem 0.8rem', borderRadius: '0.5rem', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
              <RefreshCw size={14} className={loading ? 'spin' : ''} /> Actualiser
            </button>
          </div>

          {alerts.length === 0 ? (
            <div style={{ textAlign: 'center', padding: '3rem', color: '#9ca3af' }}>
              <CheckCircle size={40} color="#10b981" style={{ marginBottom: '0.5rem' }} />
              <p>Aucune alerte critique enregistrée pour le moment.</p>
            </div>
          ) : (
            <table className="custom-table">
              <thead>
                <tr>
                  <th>ID Alerte</th>
                  <th>Titre</th>
                  <th>Description</th>
                  <th>Sévérité</th>
                  <th>Statut</th>
                  <th>Date / Heure</th>
                </tr>
              </thead>
              <tbody>
                {alerts.map((alert) => (
                  <tr key={alert.id}>
                    <td className="mono" style={{ color: '#9ca3af' }}>{alert.id.substring(0, 8)}...</td>
                    <td style={{ fontWeight: 600 }}>{alert.title || 'Tentative Suspecte'}</td>
                    <td style={{ color: '#9ca3af' }}>{alert.description}</td>
                    <td>
                      <span className={`badge badge-${(alert.severity || 'high').toLowerCase()}`}>
                        {alert.severity}
                      </span>
                    </td>
                    <td>
                      <span className="badge badge-open">{alert.status}</span>
                    </td>
                    <td className="mono" style={{ color: '#9ca3af' }}>{alert.created_at || 'À l\'instant'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </main>
    </div>
  );
}
