FROM python:3.12-slim

WORKDIR /app

# Install system deps
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Python deps first for caching
COPY ml/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy proto and generate stubs
COPY proto/ ./proto/
RUN python -m grpc_tools.protoc \
    -I proto \
    --python_out=. \
    --grpc_python_out=. \
    --pyi_out=. \
    proto/ml_service.proto

# Copy ML service code
COPY ml/cortex_ml/ ./cortex_ml/

# Move generated proto stubs into the package
RUN mv ml_service_pb2*.py* cortex_ml/proto/ 2>/dev/null || true

EXPOSE 50051

CMD ["python", "-m", "cortex_ml.server"]
