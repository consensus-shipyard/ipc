# Deployment Guide for GCP

This guide walks you through deploying the IPC tFIL faucet on Google Cloud Platform.

## Prerequisites

- GCP account with billing enabled
- `gcloud` CLI installed and configured
- Basic knowledge of GCP Compute Engine

## Quick Deployment

### 1. Create a GCP VM Instance

```bash
# Create a VM instance
gcloud compute instances create ipc-faucet \
  --zone=us-central1-a \
  --machine-type=e2-small \
  --image-family=ubuntu-2204-lts \
  --image-project=ubuntu-os-cloud \
  --boot-disk-size=20GB \
  --tags=http-server,https-server,faucet-server
```

### 2. SSH into the VM

```bash
gcloud compute ssh ipc-faucet --zone=us-central1-a
```

### 3. Install Dependencies

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Install Git
sudo apt install -y git

# Log out and back in
exit
```

### 4. Clone and Configure

```bash
# SSH back in
gcloud compute ssh ipc-faucet --zone=us-central1-a

# Clone the repository
git clone https://github.com/consensus-shipyard/ipc.git
cd ipc/faucet

# Create .env file
nano .env
```

Add your configuration:
```bash
PRIVATE_KEY=0xYOUR_PRIVATE_KEY_HERE
RPC_URL=http://node-1.test.ipc.space:8545
FAUCET_AMOUNT=1
RATE_LIMIT_WINDOW=86400000
RATE_LIMIT_MAX=1
PORT=3001
ENABLE_CORS=false
SERVE_STATIC=true
```

Save with `Ctrl+X`, then `Y`, then `Enter`.

### 5. Configure Firewall

```bash
# Create firewall rule for port 3001
gcloud compute firewall-rules create allow-ipc-faucet \
  --allow tcp:3001 \
  --source-ranges 0.0.0.0/0 \
  --target-tags faucet-server \
  --description "Allow access to IPC faucet on port 3001"
```

### 6. Deploy the Faucet

```bash
# Build and start
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f
```

### 7. Verify Deployment

```bash
# Get external IP
EXTERNAL_IP=$(gcloud compute instances describe ipc-faucet --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
echo "Faucet URL: http://$EXTERNAL_IP:3001"

# Test health endpoint
curl http://$EXTERNAL_IP:3001/api/health
```

Open the URL in your browser!

## Production Setup with HTTPS

### 1. Set Up Domain

Point your domain to the VM's external IP:
```bash
# Get external IP
gcloud compute instances describe ipc-faucet --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)'
```

Create an A record pointing to this IP.

### 2. Install Nginx and Certbot

```bash
sudo apt update
sudo apt install -y nginx certbot python3-certbot-nginx
```

### 3. Configure Nginx

```bash
# Copy the example config
sudo cp nginx.conf.example /etc/nginx/sites-available/ipc-faucet

# Edit and replace YOUR_DOMAIN
sudo nano /etc/nginx/sites-available/ipc-faucet

# Enable the site
sudo ln -s /etc/nginx/sites-available/ipc-faucet /etc/nginx/sites-enabled/

# Test configuration
sudo nginx -t

# Reload nginx
sudo systemctl reload nginx
```

### 4. Get SSL Certificate

```bash
sudo certbot --nginx -d your-domain.com
```

Follow the prompts. Certbot will automatically configure SSL.

### 5. Update Firewall for HTTPS

```bash
# The http-server and https-server tags should already allow 80/443
# If not, create rules:
gcloud compute firewall-rules create allow-http \
  --allow tcp:80 \
  --target-tags http-server

gcloud compute firewall-rules create allow-https \
  --allow tcp:443 \
  --target-tags https-server
```

### 6. Test HTTPS

Visit `https://your-domain.com` in your browser!

## Monitoring and Maintenance

### Set Up Monitoring

```bash
# Install monitoring script
cd ~/ipc/faucet
cat > monitor-faucet.sh << 'EOF'
#!/bin/bash
LOGFILE="/home/$USER/faucet-monitor.log"
cd /home/$USER/ipc/faucet

echo "=== Faucet Monitor $(date) ===" >> $LOGFILE

# Check if container is running
if docker-compose ps | grep -q "Up"; then
    echo "Status: Running" >> $LOGFILE
else
    echo "Status: DOWN - Restarting..." >> $LOGFILE
    docker-compose up -d >> $LOGFILE 2>&1
fi

# Check balance
docker-compose logs | grep "Faucet balance" | tail -1 >> $LOGFILE

# Check for errors
ERROR_COUNT=$(docker-compose logs --tail=100 | grep -c "Error")
echo "Recent errors: $ERROR_COUNT" >> $LOGFILE

echo "" >> $LOGFILE
EOF

chmod +x monitor-faucet.sh
```

### Set Up Cron Job

```bash
# Edit crontab
crontab -e

# Add these lines:
# Check faucet status every hour
0 * * * * /home/$USER/ipc/faucet/monitor-faucet.sh

# Restart faucet daily at 3 AM (optional)
0 3 * * * cd /home/$USER/ipc/faucet && docker-compose restart
```

### View Logs

```bash
# Real-time logs
docker-compose logs -f

# Last 100 lines
docker-compose logs --tail=100

# Monitor log
tail -f ~/faucet-monitor.log
```

### Check Balance

```bash
cd ~/ipc/faucet
cd scripts && npm install && cd ..
node scripts/check-balance.js
```

## Backup and Recovery

### Backup Configuration

```bash
# Backup .env file
cp ~/ipc/faucet/.env ~/ipc-faucet-backup.env

# Store securely (not on the same VM!)
gcloud compute scp ~/ipc-faucet-backup.env your-local-machine:~/backups/
```

### Update Deployment

```bash
cd ~/ipc/faucet
git pull
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

### Disaster Recovery

If the VM fails:

1. Create a new VM following steps 1-3
2. Restore your `.env` file
3. Deploy as per steps 4-6

## Cost Optimization

### Recommended Instance Types

- **e2-micro** ($5-7/month): Good for low traffic (< 100 requests/day)
- **e2-small** ($13-15/month): Recommended for moderate traffic
- **e2-medium** ($25-30/month): High traffic

### Set Up Budget Alerts

```bash
# Create budget alert (via GCP Console recommended)
# Compute Engine > Budgets & Alerts
# Set alert at 50%, 90%, 100% of budget
```

### Auto-shutdown for Testing

```bash
# Stop VM when not needed
gcloud compute instances stop ipc-faucet --zone=us-central1-a

# Start when needed
gcloud compute instances start ipc-faucet --zone=us-central1-a
```

## Security Best Practices

### 1. Restrict SSH Access

```bash
# Update firewall to allow SSH only from your IP
gcloud compute firewall-rules create allow-ssh-restricted \
  --allow tcp:22 \
  --source-ranges YOUR_IP_ADDRESS/32 \
  --target-tags faucet-server
```

### 2. Enable OS Login

```bash
gcloud compute instances add-metadata ipc-faucet \
  --zone=us-central1-a \
  --metadata enable-oslogin=TRUE
```

### 3. Regular Updates

```bash
# Set up automatic security updates
sudo apt install -y unattended-upgrades
sudo dpkg-reconfigure -plow unattended-upgrades
```

### 4. Rotate Private Key

Periodically rotate your faucet wallet:
1. Generate new wallet
2. Transfer remaining funds to new wallet
3. Update `.env` with new private key
4. Restart: `docker-compose restart`

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker-compose logs

# Rebuild
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

### Out of Memory

```bash
# Check memory usage
free -h

# Increase swap
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

### High CPU Usage

```bash
# Check container stats
docker stats

# Scale down if needed
# Consider rate limiting or smaller instance
```

## Useful Commands

```bash
# Restart faucet
docker-compose restart

# View real-time logs
docker-compose logs -f

# Check container status
docker-compose ps

# Stop faucet
docker-compose down

# Start faucet
docker-compose up -d

# Update and restart
git pull && docker-compose down && docker-compose build --no-cache && docker-compose up -d

# Check disk space
df -h

# Clean up Docker
docker system prune -a
```

## Support

For issues or questions:
- Check logs: `docker-compose logs -f`
- Review README.md
- Check IPC documentation: https://docs.ipc.space

---

**Your faucet should now be production-ready on GCP! 🚀**

