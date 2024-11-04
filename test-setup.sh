#!/bin/bash

# Exit on any error
set -e

echo "Starting server setup..."

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install Rust if not installed
if ! command_exists cargo; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust is already installed"
fi

# Install simple-http-server if not installed
if ! command_exists simple-http-server; then
    echo "Installing simple-http-server..."
    cargo install simple-http-server
else
    echo "simple-http-server is already installed"
fi

# Create directories if they don't exist
mkdir -p apps/static-site
mkdir -p libs
mkdir -p configs
mkdir -p logs

# Create a sample HTML file
cat > apps/static-site/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Test Site</title>
</head>
<body>
    <h1>Welcome to Test Site</h1>
    <p>This is a sample page served by simple-http-server.</p>
</body>
</html>
EOF

# Check rust-rxpy status
if [ ! -d "libs/rust-rxpy" ]; then
    echo "Cloning rust-rxpy..."
    cd libs
    git clone https://github.com/rust-rxpy/rust-rxpy.git
    cd rust-rxpy
    cargo build --release
    cd ../..
elif [ ! -f "libs/rust-rxpy/target/release/rxpy" ]; then
    echo "Building rust-rxpy..."
    cd libs/rust-rxpy
    cargo build --release
    cd ../..
else
    echo "rust-rxpy is already built"
fi

# Create symlink to rxpy
if [ ! -f "/usr/local/bin/rxpy" ]; then
    echo "Creating symlink for rxpy..."
    sudo ln -s "$(pwd)/libs/rust-rxpy/target/release/rxpy" /usr/local/bin/rxpy
fi

# Create rxpt-config.toml
cat > configs/rxpt-config.toml << 'EOF'
listen_port = 80

default_app = "app1"

[apps.app1]
server_name = "test.hmziq.rs"
reverse_proxy = [{ upstream = [{ location = '127.0.0.1:2345' }] }]
EOF

# Kill existing processes if running
pkill simple-http-server || true
pkill rxpy || true

# Start simple-http-server in background
echo "Starting simple-http-server..."
cd apps/static-site
nohup simple-http-server -p 2345 > ../../logs/simple-http-server.log 2>&1 &

# Start rxpy in background
echo "Starting rxpy..."
cd ../../
nohup rxpy --config configs/rxpt-config.toml > logs/rxpy.log 2>&1 &

echo "Setup completed successfully!"
echo "You can monitor the logs with:"
echo "tail -f logs/simple-http-server.log"
echo "tail -f logs/rxpy.log"
