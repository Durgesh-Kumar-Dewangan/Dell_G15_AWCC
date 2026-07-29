#!/bin/bash

# Dell G15 AWCC Web Frontend - Installation Script
# For Ubuntu 20.04 LTS and newer
# Usage: bash install.sh

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_header() {
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC} $1"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        return 1
    fi
    return 0
}

# Main installation
print_header "Dell G15 AWCC Web Frontend - Installation"

# Check if running on Ubuntu
if ! [ -f /etc/lsb-release ]; then
    print_error "This script is designed for Ubuntu/Debian systems"
    exit 1
fi

# Get Ubuntu version
. /etc/lsb-release
print_success "Detected: $DISTRIB_DESCRIPTION"

# Check prerequisites
print_header "Checking prerequisites"

# Check for Node.js
if check_command node; then
    NODE_VERSION=$(node --version)
    NODE_MAJOR=$(echo $NODE_VERSION | cut -d'.' -f1 | sed 's/v//')
    
    if [ "$NODE_MAJOR" -lt 18 ]; then
        print_warning "Node.js version $NODE_VERSION detected, but 18+ required"
        print_warning "Updating Node.js..."
        curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
        sudo apt-get install -y nodejs
        print_success "Node.js updated"
    else
        print_success "Node.js $NODE_VERSION found"
    fi
else
    print_warning "Node.js not found, installing..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
    print_success "Node.js installed"
fi

# Check for npm
if check_command npm; then
    print_success "npm $(npm --version) found"
else
    print_error "npm not found after Node.js installation"
    exit 1
fi

# Check for git
if check_command git; then
    print_success "git found"
else
    print_warning "git not found, installing..."
    sudo apt-get install -y git
    print_success "git installed"
fi

# Check for D-Bus
if check_command dbus-send; then
    print_success "D-Bus found"
else
    print_warning "D-Bus not found, installing..."
    sudo apt-get install -y dbus libdbus-1-dev
    print_success "D-Bus installed"
fi

# Check for g15-fancontrold
print_header "Checking g15-fancontrold daemon"

if check_command g15-fancontrold; then
    print_success "g15-fancontrold found"
    
    # Check if daemon is running
    if systemctl is-active --quiet g15-fancontrold; then
        print_success "g15-fancontrold is running"
    else
        print_warning "g15-fancontrold is installed but not running"
        read -p "Start g15-fancontrold now? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sudo systemctl start g15-fancontrold
            sudo systemctl enable g15-fancontrold
            print_success "g15-fancontrold started and enabled"
        fi
    fi
else
    print_warning "g15-fancontrold not found"
    print_warning "Please install g15-fancontrold from parent directory first"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Install web frontend
print_header "Installing web frontend"

# Check if in web directory
if [ ! -f "package.json" ]; then
    print_error "package.json not found"
    print_error "Please run this script from the web directory"
    exit 1
fi

# Install dependencies
print_warning "Installing npm packages (this may take a few minutes)..."
npm install
print_success "Dependencies installed"

# Build for production
print_warning "Building for production..."
npm run build
print_success "Build complete"

# Systemd service setup
print_header "Setting up systemd service"

read -p "Install systemd service for auto-start? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
    
    if [ -f "$SCRIPT_DIR/systemd/g15-awcc-web.service" ]; then
        print_warning "Installing systemd service..."
        
        # Read systemd file and replace paths
        SERVICE_FILE=$(cat "$SCRIPT_DIR/systemd/g15-awcc-web.service")
        SERVICE_FILE="${SERVICE_FILE//%u/$USER}"
        SERVICE_FILE="${SERVICE_FILE//%h/$HOME}"
        
        echo "$SERVICE_FILE" | sudo tee /etc/systemd/system/g15-awcc-web.service > /dev/null
        
        sudo systemctl daemon-reload
        sudo systemctl enable g15-awcc-web
        sudo systemctl start g15-awcc-web
        
        print_success "Systemd service installed and started"
        print_success "View logs: journalctl -u g15-awcc-web -f"
    else
        print_warning "Systemd service file not found"
    fi
fi

# Verification
print_header "Installation Summary"
print_success "Web frontend installed successfully"

if systemctl is-active --quiet g15-awcc-web; then
    print_success "Service is running"
    echo ""
    echo -e "${GREEN}Access the dashboard at:${NC}"
    echo -e "  Local:  ${BLUE}http://localhost:3000${NC}"
    echo -e "  Remote: ${BLUE}http://$(hostname -I | awk '{print $1}'):3000${NC}"
else
    print_warning "Service not started automatically"
    echo ""
    echo -e "${YELLOW}To start the web frontend:${NC}"
    echo -e "  ${BLUE}cd $(pwd)${NC}"
    echo -e "  ${BLUE}npm start${NC}"
    echo ""
    echo -e "Or to run in development mode:"
    echo -e "  ${BLUE}npm run dev${NC}"
fi

echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  1. Open your browser and navigate to http://localhost:3000"
echo "  2. Monitor temperatures and control fans"
echo "  3. View logs: journalctl -u g15-awcc-web -f"
echo "  4. For troubleshooting: see README.md"
echo ""
print_success "Installation complete!"
