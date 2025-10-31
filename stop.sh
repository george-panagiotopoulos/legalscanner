#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║       Legal Scanner - Stop Script         ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

# Load environment variables
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Default ports if not set
API_PORT=${API_PORT:-5301}
UI_PORT=${UI_PORT:-5300}
FOSSOLOGY_PORT=${FOSSOLOGY_PORT:-5302}

# Function to kill process on a port
kill_port() {
    local port=$1
    local pids=$(lsof -ti:$port 2>/dev/null || true)

    if [ ! -z "$pids" ]; then
        echo -e "${YELLOW}⚠${NC} Killing processes on port $port: $pids"
        kill -9 $pids 2>/dev/null || true
        sleep 1
        echo -e "${GREEN}✓${NC} Port $port cleared"
    else
        echo -e "${GREEN}✓${NC} Port $port is already free"
    fi
}

# Stop Docker containers
echo -e "${BLUE}Stopping Docker containers...${NC}"
docker-compose down
echo -e "${GREEN}✓${NC} Containers stopped"
echo ""

# Kill processes on ports
echo -e "${BLUE}Cleaning up ports...${NC}"
kill_port $API_PORT
kill_port $UI_PORT
kill_port $FOSSOLOGY_PORT
echo ""

# Optional: Remove volumes (ask user)
read -p "Do you want to remove Docker volumes (database data will be lost)? [y/N] " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}Removing Docker volumes...${NC}"
    docker-compose down -v
    echo -e "${GREEN}✓${NC} Volumes removed"
else
    echo -e "${YELLOW}⚠${NC} Volumes kept (database data preserved)"
fi
echo ""

echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║       All services stopped!                ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}To start again:${NC}"
echo -e "  ${YELLOW}./start.sh${NC}"
echo ""
