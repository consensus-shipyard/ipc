.PHONY: help install dev build start stop restart logs clean docker-build docker-up docker-down docker-logs generate-wallet check-balance

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

install: ## Install all dependencies
	@echo "📦 Installing dependencies..."
	npm install
	cd frontend && npm install
	cd backend && npm install
	cd scripts && npm install

dev: ## Start development servers
	@echo "🚀 Starting development servers..."
	npm run dev

build: ## Build frontend for production
	@echo "🔨 Building frontend..."
	cd frontend && npm run build

start: ## Start backend in production mode
	@echo "▶️  Starting backend..."
	cd backend && npm start

stop: ## Stop all processes
	@echo "⏹️  Stopping processes..."
	@pkill -f "node.*src/index.js" || true
	@pkill -f "vite" || true

restart: stop start ## Restart the application

logs: ## View application logs (requires Docker)
	docker-compose logs -f

clean: ## Clean build artifacts and dependencies
	@echo "🧹 Cleaning..."
	rm -rf node_modules
	rm -rf frontend/node_modules
	rm -rf frontend/dist
	rm -rf backend/node_modules
	rm -rf scripts/node_modules
	rm -rf logs

docker-build: ## Build Docker image
	@echo "🐳 Building Docker image..."
	docker-compose build

docker-up: ## Start Docker containers
	@echo "🐳 Starting Docker containers..."
	docker-compose up -d

docker-down: ## Stop Docker containers
	@echo "🐳 Stopping Docker containers..."
	docker-compose down

docker-logs: ## View Docker logs
	docker-compose logs -f

docker-restart: docker-down docker-up ## Restart Docker containers

docker-rebuild: docker-down ## Rebuild and restart Docker containers
	@echo "🐳 Rebuilding Docker containers..."
	docker-compose build --no-cache
	docker-compose up -d

generate-wallet: ## Generate a new wallet for the faucet
	@echo "🔐 Generating new wallet..."
	@cd scripts && npm install > /dev/null 2>&1 && node generate-wallet.js

check-balance: ## Check faucet wallet balance
	@echo "💰 Checking faucet balance..."
	@cd scripts && npm install > /dev/null 2>&1 && node check-balance.js

setup: install generate-wallet ## Initial setup (install deps and generate wallet)
	@echo ""
	@echo "✅ Setup complete!"
	@echo ""
	@echo "Next steps:"
	@echo "1. Fund the generated wallet address with tFIL"
	@echo "2. Copy the private key to .env file"
	@echo "3. Run 'make dev' for development or 'make docker-up' for production"
	@echo ""

test-health: ## Test faucet health endpoint
	@curl -s http://localhost:3001/api/health | json_pp || curl -s http://localhost:3001/api/health

test-config: ## Test faucet config endpoint
	@curl -s http://localhost:3001/api/config | json_pp || curl -s http://localhost:3001/api/config

status: ## Show faucet status
	@echo "📊 Faucet Status"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "Docker Containers:"
	@docker-compose ps || echo "  Docker not running or not configured"
	@echo ""
	@echo "Health Check:"
	@curl -s http://localhost:3001/api/health | json_pp || echo "  Service not responding"

