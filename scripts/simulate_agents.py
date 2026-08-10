import time
import random
import requests

API_URL = "http://localhost:3000/api/events"

AGENTS = ["agent-001", "agent-002", "agent-003"]
SOURCES = ["192.168.1.10", "192.168.1.50", "10.0.0.15", "45.33.32.156"]

EVENT_TYPES = [
    ("SUCCESSFUL_LOGIN", "LOW"),
    ("PROCESS_CREATION", "LOW"),
    ("NETWORK_CONN", "MEDIUM"),
    ("FAILED_LOGIN", "HIGH"),
    ("FAILED_LOGIN", "CRITICAL"),
]

def generate_event():
    agent = random.choice(AGENTS)
    source = random.choice(SOURCES)
    event_type, severity = random.choice(EVENT_TYPES)
    
    raw_data = {
        "user": random.choice(["admin", "root", "user1", "devops"]),
        "attempts": random.randint(1, 12),
        "process": random.choice(["powershell.exe", "sshd", "cmd.exe", "nginx"]),
    }
    
    return {
        "agent_id": agent,
        "event_type": event_type,
        "source": source,
        "severity": severity,
        "raw_data": raw_data
    }

def main():
    print("🚀 Démarrage du Simulateur d'Agents SIEM (Ctrl+C pour arrêter)...")
    count = 0
    try:
        while True:
            payload = generate_event()
            try:
                res = requests.post(API_URL, json=payload, timeout=2)
                count += 1
                icon = "🔥" if payload["severity"] in ["HIGH", "CRITICAL"] else "⚡"
                print(f"[{count}] {icon} Sent {payload['event_type']} ({payload['severity']}) from {payload['agent_id']} -> Status {res.status_code}")
            except Exception as e:
                print(f"❌ Erreur d'envoi: {e}")
            
            time.sleep(random.uniform(0.5, 2.0))
    except KeyboardInterrupt:
        print("\n⏹️ Simulation arrêtée.")

if __name__ == "__main__":
    main()
