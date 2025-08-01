#!/bin/bash
# @Version: 3.0.0
# @Author: DevOps Team
# @Description: Advanced system monitoring with notifications
# @Dependency: curl
# @Dependency: jq

# This script demonstrates advanced features that cassh2rs can convert:
# - Dynamic configuration loading
# - Multiple notification channels
# - Process monitoring
# - Network health checks
# - Resource usage tracking

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_FILE="${CONFIG_FILE:-$SCRIPT_DIR/monitoring.conf}"
LOG_DIR="${LOG_DIR:-/var/log/monitoring}"
STATE_FILE="$LOG_DIR/monitoring.state"

# Load configuration
if [[ -f "$CONFIG_FILE" ]]; then
    source "$CONFIG_FILE"
else
    cat > "$CONFIG_FILE" <<EOF
# Monitoring Configuration
MONITOR_INTERVAL=60
CPU_THRESHOLD=80
MEMORY_THRESHOLD=90
DISK_THRESHOLD=85
ENABLE_NOTIFICATIONS=true
SLACK_WEBHOOK=""
EMAIL_RECIPIENTS=""
MONITORED_SERVICES="nginx postgresql redis"
HEALTH_CHECK_URLS="https://api.example.com/health"
EOF
    echo "Created default configuration at $CONFIG_FILE"
    exit 1
fi

# Ensure log directory exists
mkdir -p "$LOG_DIR"

# Initialize state file
init_state() {
    if [[ ! -f "$STATE_FILE" ]]; then
        cat > "$STATE_FILE" <<EOF
{
  "last_check": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "alerts": {},
  "metrics": {}
}
EOF
    fi
}

# Logging functions with levels
log() {
    local level="$1"
    shift
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [$level] $*" | tee -a "$LOG_DIR/monitoring.log"
}

log_info() { log "INFO" "$@"; }
log_warn() { log "WARN" "$@"; }
log_error() { log "ERROR" "$@"; }

# State management
update_state() {
    local key="$1"
    local value="$2"
    local temp_file=$(mktemp)
    
    jq --arg k "$key" --arg v "$value" '.[$k] = $v' "$STATE_FILE" > "$temp_file"
    mv "$temp_file" "$STATE_FILE"
}

get_state() {
    local key="$1"
    jq -r ".$key // empty" "$STATE_FILE" 2>/dev/null || echo ""
}

# Notification system
send_notification() {
    local title="$1"
    local message="$2"
    local severity="${3:-info}"
    
    if [[ "$ENABLE_NOTIFICATIONS" != "true" ]]; then
        return
    fi
    
    # Desktop notification
    if command -v notify-send &> /dev/null; then
        notify-send -u "$severity" "$title" "$message"
    fi
    
    # Slack notification
    if [[ -n "$SLACK_WEBHOOK" ]]; then
        local color="good"
        case "$severity" in
            critical) color="danger" ;;
            warning) color="warning" ;;
        esac
        
        curl -s -X POST "$SLACK_WEBHOOK" \
            -H "Content-Type: application/json" \
            -d @- <<EOF
{
  "attachments": [{
    "color": "$color",
    "title": "$title",
    "text": "$message",
    "footer": "$(hostname)",
    "ts": $(date +%s)
  }]
}
EOF
    fi
    
    # Email notification
    if [[ -n "$EMAIL_RECIPIENTS" ]]; then
        echo "$message" | mail -s "[$severity] $title" "$EMAIL_RECIPIENTS"
    fi
    
    # Log notification
    log_info "Notification sent: [$severity] $title - $message"
}

# System metrics collection
get_cpu_usage() {
    local cpu_idle=$(top -bn1 | grep "Cpu(s)" | awk '{print $8}' | cut -d'%' -f1)
    echo "$((100 - ${cpu_idle%.*}))"
}

get_memory_usage() {
    free | awk 'NR==2{printf "%.0f", $3*100/$2}'
}

get_disk_usage() {
    local mount_point="${1:-/}"
    df -h "$mount_point" | awk 'NR==2{print $5}' | sed 's/%//'
}

get_load_average() {
    uptime | awk -F'load average:' '{print $2}' | xargs
}

# Process monitoring
check_process() {
    local process="$1"
    
    if pgrep -x "$process" > /dev/null; then
        return 0
    else
        return 1
    fi
}

monitor_services() {
    local failed_services=()
    
    for service in $MONITORED_SERVICES; do
        if ! check_process "$service"; then
            failed_services+=("$service")
            log_error "Service $service is not running"
            
            # Attempt to restart
            if command -v systemctl &> /dev/null; then
                log_info "Attempting to restart $service"
                if systemctl restart "$service" 2>/dev/null; then
                    log_info "Successfully restarted $service"
                    send_notification "Service Restarted" "$service was down and has been restarted" "warning"
                else
                    log_error "Failed to restart $service"
                    send_notification "Service Down" "$service is down and could not be restarted" "critical"
                fi
            fi
        fi
    done
    
    if [[ ${#failed_services[@]} -eq 0 ]]; then
        log_info "All monitored services are running"
    fi
}

# Network health checks
check_health_endpoints() {
    local failed_checks=()
    
    for url in $HEALTH_CHECK_URLS; do
        log_info "Checking health endpoint: $url"
        
        local response=$(curl -s -o /dev/null -w "%{http_code}" -m 10 "$url" || echo "000")
        
        if [[ "$response" != "200" ]]; then
            failed_checks+=("$url (HTTP $response)")
            log_error "Health check failed for $url: HTTP $response"
        fi
    done
    
    if [[ ${#failed_checks[@]} -gt 0 ]]; then
        send_notification "Health Check Failed" "Failed endpoints: ${failed_checks[*]}" "critical"
    fi
}

# Resource monitoring
check_resources() {
    local alerts=()
    
    # CPU check
    local cpu_usage=$(get_cpu_usage)
    if [[ $cpu_usage -gt $CPU_THRESHOLD ]]; then
        alerts+=("CPU usage is ${cpu_usage}% (threshold: ${CPU_THRESHOLD}%)")
        update_state "metrics.cpu_usage" "$cpu_usage"
    fi
    
    # Memory check
    local mem_usage=$(get_memory_usage)
    if [[ $mem_usage -gt $MEMORY_THRESHOLD ]]; then
        alerts+=("Memory usage is ${mem_usage}% (threshold: ${MEMORY_THRESHOLD}%)")
        update_state "metrics.memory_usage" "$mem_usage"
    fi
    
    # Disk check
    local disk_usage=$(get_disk_usage)
    if [[ $disk_usage -gt $DISK_THRESHOLD ]]; then
        alerts+=("Disk usage is ${disk_usage}% (threshold: ${DISK_THRESHOLD}%)")
        update_state "metrics.disk_usage" "$disk_usage"
    fi
    
    # Send consolidated alert if needed
    if [[ ${#alerts[@]} -gt 0 ]]; then
        local alert_message=$(printf '%s\n' "${alerts[@]}")
        send_notification "Resource Alert" "$alert_message" "warning"
    else
        log_info "All resources within thresholds"
    fi
    
    # Log current metrics
    log_info "Current metrics - CPU: ${cpu_usage}%, Memory: ${mem_usage}%, Disk: ${disk_usage}%"
}

# Log rotation
rotate_logs() {
    local log_file="$LOG_DIR/monitoring.log"
    local max_size=$((10 * 1024 * 1024))  # 10MB
    
    if [[ -f "$log_file" ]]; then
        local size=$(stat -f%z "$log_file" 2>/dev/null || stat -c%s "$log_file" 2>/dev/null || echo 0)
        
        if [[ $size -gt $max_size ]]; then
            local timestamp=$(date +%Y%m%d_%H%M%S)
            mv "$log_file" "${log_file}.${timestamp}"
            gzip "${log_file}.${timestamp}"
            log_info "Rotated log file"
            
            # Clean old logs (keep last 7 days)
            find "$LOG_DIR" -name "monitoring.log.*.gz" -mtime +7 -delete
        fi
    fi
}

# Signal handling
cleanup() {
    log_info "Monitoring script terminated"
    update_state "last_check" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Main monitoring loop
main() {
    log_info "Starting monitoring system (PID: $$)"
    init_state
    
    # Run checks based on mode
    if [[ "${1:-}" == "--once" ]]; then
        # Single run mode
        log_info "Running single check"
        monitor_services
        check_health_endpoints
        check_resources
        rotate_logs
    else
        # Continuous monitoring mode
        log_info "Starting continuous monitoring (interval: ${MONITOR_INTERVAL}s)"
        
        while true; do
            log_info "Running monitoring checks"
            
            monitor_services
            check_health_endpoints
            check_resources
            rotate_logs
            
            update_state "last_check" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
            
            sleep "$MONITOR_INTERVAL"
        done
    fi
}

# Execute main function
main "$@"