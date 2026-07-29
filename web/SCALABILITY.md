# Dell G15 AWCC - Scalability Guide

Complete guide for scaling the application from single laptop to enterprise deployment.

## Scalability Levels

### Level 1: Single Device (Current State)

**Use Case**: Individual laptop monitoring

```
┌──────────────┐
│ Browser      │
└──────┬───────┘
       │ HTTP
       ▼
   Next.js App (localhost:3000)
       │ D-Bus
       ▼
  g15-fancontrold
       │
       ▼
   Hardware
```

**Setup:**
```bash
npm run build
npm start
# Access: http://localhost:3000
```

**Resources:**
- CPU: 50-100MB
- Memory: 150-300MB
- Disk: 500MB

### Level 2: Local Network

**Use Case**: Multi-device monitoring on same network

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Browser 1   │     │ Browser 2   │     │ Browser 3   │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │ HTTP              │ HTTP              │ HTTP
       └──────────────┬────┴──────────┬───────┘
                      │ LAN (192.168.x.x)
                      ▼
                  Nginx (Port 80)
                  Load Balancer
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
    Next.js:3000  3001        3002
        │             │             │ D-Bus
        └─────────────┼─────────────┘
                      ▼
                 g15-fancontrold
```

**Setup:**

```bash
# Install Nginx
sudo apt install -y nginx

# Create load balancer config
sudo tee /etc/nginx/sites-available/g15-awcc > /dev/null << 'EOF'
upstream g15_backend {
    server localhost:3000 weight=1;
    server localhost:3001 weight=1;
    server localhost:3002 weight=1;
}

server {
    listen 80;
    server_name _;
    
    location / {
        proxy_pass http://g15_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
    
    location /api/ {
        proxy_pass http://g15_backend;
        proxy_buffering off;
        proxy_request_buffering off;
    }
}
EOF

# Enable config
sudo ln -s /etc/nginx/sites-available/g15-awcc /etc/nginx/sites-enabled/default
sudo systemctl restart nginx

# Start multiple instances
PORT=3000 npm start &
PORT=3001 npm start &
PORT=3002 npm start &
```

**Access:** http://<server-ip> (from any device on network)

### Level 3: Data Center / Server

**Use Case**: Centralized monitoring for multiple devices

```
Internet
    │
    ▼
┌──────────────┐
│ CloudFlare   │ (Optional CDN)
└──────┬───────┘
       │
       ▼
┌──────────────────────────────┐
│ Reverse Proxy (Nginx/HAProxy) │
│ Port: 443 (HTTPS)             │
└──────┬───────────────────────┘
       │
       ├─────────────┬──────────────┬──────────────┐
       ▼             ▼              ▼              ▼
    Node-1       Node-2         Node-3        Node-4
    (3000)       (3001)         (3002)        (3003)

PostgreSQL Database (Central)
Redis Cache (Session Storage)
```

**Setup with PM2 Clustering:**

```bash
# Install PM2 globally
npm install -g pm2

# Create ecosystem.config.js
cat > ecosystem.config.js << 'EOF'
module.exports = {
  apps: [
    {
      name: 'g15-awcc',
      script: 'npm',
      args: 'start',
      instances: 4,
      exec_mode: 'cluster',
      error_file: './logs/err.log',
      out_file: './logs/out.log',
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z',
      max_memory_restart: '500M',
      env: {
        NODE_ENV: 'production',
        PORT: 3000,
      },
    },
  ],
};
EOF

# Start with PM2
pm2 start ecosystem.config.js

# Monitor
pm2 monit

# View logs
pm2 logs

# Auto-restart on reboot
pm2 startup
pm2 save
```

**Nginx Configuration for Multiple Servers:**

```nginx
# /etc/nginx/sites-available/g15-awcc-cluster
upstream g15_cluster {
    least_conn;  # Use least connections algorithm
    
    server localhost:3000 max_fails=3 fail_timeout=30s;
    server localhost:3001 max_fails=3 fail_timeout=30s;
    server localhost:3002 max_fails=3 fail_timeout=30s;
    server localhost:3003 max_fails=3 fail_timeout=30s;
    
    keepalive 32;
}

server {
    listen 80;
    server_name _;
    
    # Redirect to HTTPS
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name _;
    
    ssl_certificate /etc/ssl/certs/g15-awcc.crt;
    ssl_certificate_key /etc/ssl/private/g15-awcc.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    
    # Gzip compression
    gzip on;
    gzip_types text/plain text/css application/json application/javascript;
    gzip_min_length 1000;
    
    location / {
        proxy_pass http://g15_cluster;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket support
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
    
    # Cache static assets
    location /_next/static/ {
        expires 365d;
        add_header Cache-Control "public, immutable";
    }
}
```

### Level 4: Kubernetes Deployment

**Use Case**: Enterprise-grade auto-scaling

```dockerfile
# Dockerfile
FROM node:20-alpine AS deps
WORKDIR /app
COPY package*.json ./
RUN npm ci

FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
COPY . .
RUN npm ci
RUN npm run build

FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/public ./public
COPY --from=builder /app/package*.json ./
EXPOSE 3000
CMD ["npm", "start"]
```

```yaml
# kubernetes-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: g15-awcc
  labels:
    app: g15-awcc
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: g15-awcc
  template:
    metadata:
      labels:
        app: g15-awcc
    spec:
      containers:
      - name: g15-awcc
        image: dell-g15-awcc:latest
        ports:
        - containerPort: 3000
        env:
        - name: NODE_ENV
          value: "production"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: g15-awcc-service
spec:
  selector:
    app: g15-awcc
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 3000

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: g15-awcc-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: g15-awcc
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

```bash
# Deploy to Kubernetes
kubectl apply -f kubernetes-deployment.yaml

# Check deployment
kubectl get deployments
kubectl get pods
kubectl get services

# Scale manually if needed
kubectl scale deployment g15-awcc --replicas=5
```

## Performance Optimization

### 1. Build Optimization

```bash
# Enable SWC minification
npm run build -- --swcMinify

# Analyze bundle size
npm install --save-dev @next/bundle-analyzer

# In next.config.ts:
const withBundleAnalyzer = require('@next/bundle-analyzer')({
  enabled: process.env.ANALYZE === 'true',
});

export default withBundleAnalyzer({...});

# Analyze
ANALYZE=true npm run build
```

### 2. Runtime Optimization

```typescript
// Implement caching layer
import NodeCache from 'node-cache';

const cache = new NodeCache({
  stdTTL: 5,        // 5 second TTL
  checkperiod: 1,   // Check every 1 second
  useClones: true,  // Clone values
});

export async function getDaemonStatus() {
  const cached = cache.get('daemon_status');
  if (cached) return cached;
  
  const data = await callDaemon();
  cache.set('daemon_status', data);
  return data;
}
```

### 3. Database Optimization (Future)

```sql
-- Create indexes for performance
CREATE INDEX idx_temperature_timestamp ON temperature_history(timestamp DESC);
CREATE INDEX idx_device_id_timestamp ON temperature_history(device_id, timestamp DESC);

-- Partition old data
CREATE TABLE temperature_history_2024_q1 
  PARTITION OF temperature_history
  FOR VALUES FROM ('2024-01-01') TO ('2024-04-01');
```

### 4. Frontend Optimization

```typescript
// Code splitting
const TemperatureChart = dynamic(() => import('@/components/temperature-chart'), {
  loading: () => <ChartSkeleton />,
});

// Image optimization
import Image from 'next/image';

<Image
  src="/icon.png"
  width={40}
  height={40}
  priority={true}
  placeholder="blur"
/>

// CSS optimization
// Enable CSS minification in globals.css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
  }
}
```

## Monitoring and Observability

### Application Monitoring

```bash
# Install monitoring stack
npm install --save prometheus-client pino pino-http

# Create monitoring endpoint (app/api/metrics/route.ts)
import { register } from 'prom-client';

export async function GET() {
  return new Response(await register.metrics(), {
    headers: { 'Content-Type': register.contentType },
  });
}

# Access metrics
curl http://localhost:3000/api/metrics
```

### System Monitoring

```bash
# Install system monitoring
sudo apt install -y prometheus grafana-server

# Monitor with htop
htop -p $(pgrep -f "next start")

# Check resource usage
ps aux | grep "next"
free -h
df -h
```

## Security at Scale

```bash
# Rate limiting
npm install express-rate-limit

# CORS configuration
const corsOptions = {
  origin: process.env.ALLOWED_ORIGINS?.split(','),
  credentials: true,
  optionsSuccessStatus: 200,
};

# Environment variables
cp .env.example .env.production
# Add sensitive config

# SSL/TLS certificates (Let's Encrypt)
sudo apt install -y certbot python3-certbot-nginx
sudo certbot certonly --nginx -d example.com
```

## Backup and Disaster Recovery

```bash
# Automated backup script
#!/bin/bash
BACKUP_DIR="/backup/g15-awcc"
mkdir -p $BACKUP_DIR

# Backup application
tar -czf $BACKUP_DIR/app-$(date +%Y%m%d_%H%M%S).tar.gz .

# Backup database (if using)
pg_dump $DATABASE_URL > $BACKUP_DIR/db-$(date +%Y%m%d).sql

# Keep only last 30 days
find $BACKUP_DIR -type f -mtime +30 -delete

# Schedule with cron
# 0 2 * * * /usr/local/bin/backup-g15-awcc.sh
```

## Troubleshooting at Scale

```bash
# Performance debugging
node --prof app.js
node --prof-process isolate-*.log > profile.txt

# Memory leak detection
npm install clinic
clinic doctor -- npm start

# Load testing
npm install -g autocannon
autocannon http://localhost:3000 -c 100 -d 30

# Network monitoring
sudo nethogs
sudo iftop
```

## Scaling Checklist

- [ ] Database optimization (indexes, partitioning)
- [ ] Caching layer (Redis)
- [ ] Load balancer (Nginx/HAProxy)
- [ ] SSL/TLS certificates
- [ ] Monitoring and alerting
- [ ] Backup and recovery plan
- [ ] Auto-scaling policies
- [ ] Security hardening
- [ ] Performance testing
- [ ] Documentation updated

## References

- [Next.js Performance](https://nextjs.org/docs/advanced-features/measuring-performance)
- [Node.js Clustering](https://nodejs.org/en/docs/guides/nodejs-performance-on-linux-servers/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Nginx Load Balancing](https://nginx.org/en/docs/http/load_balancing.html)
