#!/bin/bash
# Production deployment script

echo "🚀 Deploying AIMF to production"

# Option 1: Direct binary install
curl -fsSL https://aimf.io/install.sh | bash

# Option 2: Docker
docker pull aimf/verifier:latest
docker run -d -p 8080:8080 aimf/verifier

# Option 3: Kubernetes
kubectl apply -f https://aimf.io/k8s/deployment.yaml

# Option 4: systemd service
sudo tee /etc/systemd/system/aimf.service <<EOF
[Unit]
Description=AIMF Verification Service
After=network.target

[Service]
ExecStart=/usr/local/bin/aimf serve --port 8080
Restart=always
User=nobody

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable aimf
sudo systemctl start aimf

echo "✅ AIMF running on http://localhost:8080"