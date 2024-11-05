#!/bin/bash

# Exit on any error
set -e

echo "Starting server setup..."

GIT_USER_NAME="USERNAME"
GIT_USER_EMAIL="email@gmail.com"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}
# Function to check if package is installed (Debian/Ubuntu)
is_pkg_installed() {
    dpkg -l "$1" | grep -q '^ii'
}

# Function to check if package group is installed (RHEL/CentOS)
is_group_installed() {
    yum grouplist installed | grep -q "$1"
}

# Install git if not installed
if ! command_exists git; then
    echo "Installing git..."
    if [ -f /etc/debian_version ]; then
        sudo apt-get update
        sudo apt-get install -y git
    elif [ -f /etc/redhat-release ]; then
        sudo yum install -y git
    elif [ -f /etc/arch-release ]; then
        sudo pacman -Sy git
    fi

    # Configure git globally after installation
    echo "Configuring git globally..."
    git config --global user.name "$GIT_USER_NAME"
    git config --global user.email "$GIT_USER_EMAIL"
else
    echo "git is already installed"

    # Check if git config exists, if not configure it
    if [ -z "$(git config --global user.name)" ] || [ -z "$(git config --global user.email)" ]; then
        echo "Configuring git globally..."
        git config --global user.name "$GIT_USER_NAME"
        git config --global user.email "$GIT_USER_EMAIL"
    else
        echo "git is already configured globally"
    fi
fi

# Install build dependencies based on distribution
if [ -f /etc/debian_version ]; then
    # Debian/Ubuntu
    if ! is_pkg_installed build-essential; then
        echo "Installing build-essential..."
        sudo apt-get update
        sudo apt-get install -y build-essential
    else
        echo "build-essential is already installed"
    fi

    if ! is_pkg_installed libssl-dev; then
        echo "Installing OpenSSL development packages..."
        sudo apt-get install -y openssl libssl-dev
    else
        echo "OpenSSL development packages are already installed"
    fi

    # Additional packages
    for pkg in pkg-config perl make gcc; do
        if ! is_pkg_installed $pkg; then
            echo "Installing $pkg..."
            sudo apt-get install -y $pkg
        else
            echo "$pkg is already installed"
        fi
    done

elif [ -f /etc/redhat-release ]; then
    # CentOS/RHEL
    if ! is_group_installed "Development Tools"; then
        echo "Installing Development Tools..."
        sudo yum groupinstall -y "Development Tools"
    else
        echo "Development Tools are already installed"
    fi

    # Check individual packages
    for pkg in gcc openssl openssl-devel perl make; do
        if ! rpm -q $pkg >/dev/null 2>&1; then
            echo "Installing $pkg..."
            sudo yum install -y $pkg
        else
            echo "$pkg is already installed"
        fi
    done

elif [ -f /etc/arch-release ]; then
    # Arch Linux
    # Note: Arch handles dependencies differently, but we'll still check
    if ! pacman -Qi base-devel >/dev/null 2>&1; then
        echo "Installing base-devel..."
        sudo pacman -Sy base-devel
    else
        echo "base-devel is already installed"
    fi

    if ! pacman -Qi openssl >/dev/null 2>&1; then
        echo "Installing openssl..."
        sudo pacman -Sy openssl
    else
        echo "openssl is already installed"
    fi
fi

# Verify OpenSSL installation
if ! command -v openssl &> /dev/null; then
    echo "OpenSSL installation failed"
    exit 1
else
    echo "OpenSSL is properly installed"
    openssl version
fi

# Source bashrc to ensure environment variables are set
if [ -f "$HOME/.bashrc" ]; then
    echo "Sourcing ~/.bashrc..."
    source "$HOME/.bashrc"
fi

# Source cargo environment specifically
if [ -f "$HOME/.cargo/env" ]; then
    echo "Sourcing cargo environment..."
    source "$HOME/.cargo/env"
fi

# Install Rust if not installed
if ! command_exists cargo; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    source "$HOME/.bashrc"
    # Reload shell environment
    exec $SHELL
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

# Check rust-rpxy status
if [ ! -d "libs/rust-rpxy" ]; then
    echo "Cloning rust-rpxy..."
    cd libs
    git clone http://github.com/junkurihara/rust-rpxy.git
    cd rust-rpxy
    git submodule update --init
    cargo build --release
    cd ../..
elif [ ! -f "libs/rust-rpxy/target/release/rpxy" ]; then
    echo "Building rust-rpxy..."
    cd libs/rust-rpxy
    git pull
    git submodule update --init
    cargo build --release
    cd ../..
else
    echo "rust-rpxy is already built"
fi

# Create symlink to rpxy
if [ ! -f "/usr/local/bin/rpxy" ]; then
    echo "Creating symlink for rpxy..."
    sudo ln -s "$(pwd)/libs/rust-rpxy/target/release/rpxy" /usr/local/bin/rpxy
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
pkill rpxy || true

# Start simple-http-server in background
echo "Starting simple-http-server..."
nohup simple-http-server -i -p 2345 ~/apps/static-site > ~/logs/simple-http-server.log 2>&1 &
nohup simple-http-server -i -p 2222 ~/apps/static-blog > ~/logs/static-blog.log 2>&1 &

# Start rpxy in background
echo "Starting rpxy..."
cd ../../
nohup rpxy --config configs/rxpt-config.toml > logs/rpxy.log 2>&1 &

nohup river --config-kdl ~/configs/river.kdl > ~/logs/river.log 2>&1 &

echo "Setup completed successfully!"
echo "You can monitor the logs with:"
echo "tail -f logs/simple-http-server.log"
echo "tail -f logs/rpxy.log"
