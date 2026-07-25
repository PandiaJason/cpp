#!/usr/bin/env bash
set -e

PORT=3030
CPP_DIR="/Users/admin/Jas Apps/Context Provider Protocol"

echo "=== Checking CPP Daemon Server on port ${PORT} ==="

if curl -s "http://localhost:${PORT}/api/rpc" -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"cpp/capabilities"}' > /dev/null 2>&1; then
    echo "✅ CPP Server is already running and responsive on http://localhost:${PORT}"
    exit 0
fi

echo "⚠️  CPP Server not detected on port ${PORT}. Spawning daemon..."

# Ensure any dangling process on port 3030 is cleared
lsof -ti:${PORT} | xargs kill -9 2>/dev/null || true

# Build and start cpp-server in background
cd "${CPP_DIR}"
cargo build --bin cpp-server --quiet
nohup cargo run --bin cpp-server --quiet > "${CPP_DIR}/.agents/cpp-server.log" 2>&1 &

# Wait for server to become healthy
for i in {1..15}; do
    if curl -s "http://localhost:${PORT}/api/rpc" -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"cpp/capabilities"}' > /dev/null 2>&1; then
        echo "🎉 CPP Server successfully started on http://localhost:${PORT} (PID $!)"
        exit 0
    fi
    sleep 0.5
done

echo "❌ Failed to verify CPP Server startup within 7.5s. Log output:"
cat "${CPP_DIR}/.agents/cpp-server.log" | tail -20
exit 1
