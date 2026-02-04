.PHONY: dev build test clean proto

# Start all services
dev:
	docker compose up --build

# Start infrastructure only (Postgres, Weaviate)
infra:
	docker compose up postgres weaviate -d

# Build Rust
build-rust:
	cd rust && cargo build

# Build Python proto stubs
proto:
	python -m grpc_tools.protoc \
		-I proto \
		--python_out=ml/cortex_ml/proto \
		--grpc_python_out=ml/cortex_ml/proto \
		--pyi_out=ml/cortex_ml/proto \
		proto/ml_service.proto

# Run Rust tests
test-rust:
	cd rust && cargo test

# Run Python tests
test-python:
	cd ml && python -m pytest tests/ -v

# Run all tests
test: test-rust test-python

# Format
fmt:
	cd rust && cargo fmt
	cd ml && ruff format .

# Lint
lint:
	cd rust && cargo clippy -- -D warnings
	cd ml && ruff check .

# Clean
clean:
	cd rust && cargo clean
	docker compose down -v

# Run Rust API locally (requires infra running)
run-api:
	cd rust && cargo run --bin cortex-api

# Run Python ML service locally
run-ml:
	cd ml && python -m cortex_ml.server
