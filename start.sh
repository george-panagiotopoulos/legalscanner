#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║       Legal Scanner - Start Script        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

# Load environment variables
if [ -f .env ]; then
    echo -e "${GREEN}✓${NC} Loading environment variables from .env"
    export $(grep -v '^#' .env | xargs)
else
    echo -e "${RED}✗${NC} .env file not found! Creating from .env.example..."
    if [ -f .env.example ]; then
        cp .env.example .env
        export $(grep -v '^#' .env | xargs)
        echo -e "${YELLOW}⚠${NC} Please edit .env file with your configuration"
    else
        echo -e "${RED}✗${NC} .env.example not found! Cannot continue."
        exit 1
    fi
fi

# Default ports if not set in .env
API_PORT=${API_PORT:-5301}
UI_PORT=${UI_PORT:-5300}
FOSSOLOGY_PORT=${FOSSOLOGY_PORT:-5302}

echo -e "${BLUE}Port Configuration:${NC}"
echo -e "  API Port:       ${GREEN}${API_PORT}${NC}"
echo -e "  UI Port:        ${GREEN}${UI_PORT}${NC}"
echo -e "  Fossology Port: ${GREEN}${FOSSOLOGY_PORT}${NC}"
echo ""

# Function to kill process on a port
kill_port() {
    local port=$1
    local pids=$(lsof -ti:$port 2>/dev/null || true)

    if [ ! -z "$pids" ]; then
        echo -e "${YELLOW}⚠${NC} Port $port is in use. Killing processes: $pids"
        kill -9 $pids 2>/dev/null || true
        sleep 1
        echo -e "${GREEN}✓${NC} Port $port cleared"
    fi
}

# Clean ports
echo -e "${BLUE}Checking and cleaning ports...${NC}"
kill_port $API_PORT
kill_port $UI_PORT
kill_port $FOSSOLOGY_PORT

# Stop any running containers
echo -e "${BLUE}Stopping existing containers...${NC}"
docker-compose down 2>/dev/null || true
echo -e "${GREEN}✓${NC} Containers stopped"
echo ""

# Create necessary directories
echo -e "${BLUE}Creating directories...${NC}"
mkdir -p data
mkdir -p ${TEMP_WORKSPACE_DIR:-/tmp/legalscanner}
echo -e "${GREEN}✓${NC} Directories created"
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}✗${NC} Docker is not running. Please start Docker Desktop and try again."
    exit 1
fi
echo -e "${GREEN}✓${NC} Docker is running"
echo ""

# Start services with Docker Compose
echo -e "${BLUE}Starting services with Docker Compose...${NC}"
echo -e "${YELLOW}This may take a few minutes on first run (building images)...${NC}"
echo ""

docker-compose up --build -d

echo ""
echo -e "${GREEN}✓${NC} Services started!"
echo ""

# Wait for services to be healthy
echo -e "${BLUE}Waiting for services to be ready...${NC}"
echo -e "${YELLOW}Note: Fossology may take 2-5 minutes to fully initialize${NC}"
echo ""

# Wait for API
echo -n "Waiting for API... "
for i in {1..30}; do
    if curl -s http://localhost:${API_PORT}/health > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        break
    fi
    sleep 2
    echo -n "."
done

# Check if UI is up
echo -n "Waiting for UI...  "
for i in {1..15}; do
    if curl -s http://localhost:${UI_PORT} > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        break
    fi
    sleep 2
    echo -n "."
done

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          Services are running!             ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Access URLs:${NC}"
echo -e "  API:       ${GREEN}http://localhost:${API_PORT}${NC}"
echo -e "  API Health: ${GREEN}http://localhost:${API_PORT}/health${NC}"
echo -e "  UI:        ${GREEN}http://localhost:${UI_PORT}${NC}"
echo -e "  Fossology: ${GREEN}http://localhost:${FOSSOLOGY_PORT}${NC}"
echo ""
echo -e "${BLUE}View logs:${NC}"
echo -e "  All services: ${YELLOW}docker-compose logs -f${NC}"
echo -e "  API only:     ${YELLOW}docker-compose logs -f api${NC}"
echo -e "  UI only:      ${YELLOW}docker-compose logs -f ui${NC}"
echo ""
echo -e "${BLUE}Stop services:${NC}"
echo -e "  ${YELLOW}./stop.sh${NC}"
echo ""
