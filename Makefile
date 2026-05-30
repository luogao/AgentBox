.PHONY: build setup up down clean logs

# Build everything: Rust release + Docker images
build:
	cargo build --release
	docker compose build

# One-command setup: build + start all services
setup: build up

# Start all services
up:
	docker compose up -d

# Stop all services
down:
	docker compose down

# View logs
logs:
	docker compose logs -f

# Clean up: stop services, remove volumes, remove images
clean:
	docker compose down -v
	-docker rmi agent-sandbox:latest agentbox-control:latest 2>/dev/null

# Development: start control-plane + admin-ui locally
dev:
	@echo "Starting control-plane on :8080 and admin-ui on :5173..."
	@mkdir -p data
	@trap 'kill %1 %2 2>/dev/null' EXIT; \
	DATABASE_URL="sqlite:./data/agent_sandbox.db?mode=rwc" RUST_LOG=info cargo run -p control-plane & \
	cd admin-ui && npm run dev & \
	wait
