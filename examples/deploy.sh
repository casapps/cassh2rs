#!/bin/bash
# @Version: 1.0.0
# @Author: DevOps Team
# @Description: Deployment script with notifications
# @Dependency: git
# @Dependency: docker

set -e

# Configuration
APP_NAME="myapp"
DEPLOY_DIR="/opt/${APP_NAME}"
GIT_REPO="https://github.com/example/myapp.git"
DOCKER_IMAGE="${APP_NAME}:latest"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

send_notification() {
    local title="$1"
    local message="$2"
    
    # Send desktop notification if available
    if command -v notify-send &> /dev/null; then
        notify-send "$title" "$message"
    fi
    
    # Log to file
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $title: $message" >> /var/log/${APP_NAME}.log
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing_deps=()
    
    # Check for required commands
    for cmd in git docker; do
        if ! command -v $cmd &> /dev/null; then
            missing_deps+=($cmd)
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        exit 1
    fi
    
    # Check if deploy directory exists
    if [ ! -d "$DEPLOY_DIR" ]; then
        log_warning "Deploy directory does not exist, creating..."
        mkdir -p "$DEPLOY_DIR"
    fi
    
    log_info "All prerequisites met"
}

update_code() {
    log_info "Updating code from repository..."
    
    cd "$DEPLOY_DIR"
    
    if [ -d ".git" ]; then
        # Repository exists, pull latest
        git pull origin main
    else
        # Clone repository
        git clone "$GIT_REPO" .
    fi
    
    log_info "Code updated successfully"
}

build_docker_image() {
    log_info "Building Docker image..."
    
    cd "$DEPLOY_DIR"
    
    if [ ! -f "Dockerfile" ]; then
        log_error "Dockerfile not found!"
        return 1
    fi
    
    docker build -t "$DOCKER_IMAGE" .
    
    log_info "Docker image built successfully"
}

deploy_application() {
    log_info "Deploying application..."
    
    # Stop existing container if running
    if docker ps -a | grep -q "$APP_NAME"; then
        log_info "Stopping existing container..."
        docker stop "$APP_NAME" || true
        docker rm "$APP_NAME" || true
    fi
    
    # Run new container
    docker run -d \
        --name "$APP_NAME" \
        --restart unless-stopped \
        -p 8080:8080 \
        "$DOCKER_IMAGE"
    
    # Wait for application to start
    sleep 5
    
    # Check if container is running
    if docker ps | grep -q "$APP_NAME"; then
        log_info "Application deployed successfully"
        return 0
    else
        log_error "Failed to start application"
        return 1
    fi
}

cleanup() {
    log_info "Cleaning up old Docker images..."
    docker image prune -f
}

# Main execution
main() {
    log_info "Starting deployment of $APP_NAME"
    send_notification "Deployment Started" "Beginning deployment of $APP_NAME"
    
    # Run deployment steps
    check_prerequisites
    update_code
    build_docker_image
    
    if deploy_application; then
        cleanup
        send_notification "Deployment Success" "$APP_NAME deployed successfully!"
        log_info "Deployment completed successfully"
        exit 0
    else
        send_notification "Deployment Failed" "Failed to deploy $APP_NAME"
        log_error "Deployment failed"
        exit 1
    fi
}

# Execute main function
main "$@"