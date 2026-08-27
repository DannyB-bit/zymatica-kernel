FROM python:3.11-slim

LABEL maintainer="Zymatica <zymatica.space>"
LABEL description="ZK-LoRaWAN Gateway — LoRa RF to Solana bridge"

WORKDIR /app

# System dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    libffi-dev \
    && rm -rf /var/lib/apt/lists/*

# Python dependencies
COPY requirements-gateway.txt ./
RUN pip install --no-cache-dir -r requirements-gateway.txt

# Copy gateway code + all modules
COPY gateway/ ./gateway/
COPY semantic_codec/ ./semantic_codec/
COPY fec/ ./fec/
COPY proof_compression/ ./proof_compression/
COPY semantic_gating/ ./semantic_gating/
COPY reputation/ ./reputation/
COPY vk_compression/ ./vk_compression/

# Environment
ENV PYTHONUNBUFFERED=1
ENV GATEWAY_MODE=batch
ENV GATEWAY_BATCH_SIZE=100
ENV GATEWAY_BATCH_TIMEOUT=900
ENV SOLANA_RPC=https://api.devnet.solana.com
ENV LORA_FREQUENCY=915000000
ENV LORA_SF=12

EXPOSE 8080

CMD ["python", "-m", "gateway.gateway"]
