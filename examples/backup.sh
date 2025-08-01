#!/bin/bash
# @Version: 2.1.0
# @Author: SysAdmin
# @Description: Backup script with compression and rotation

# Configuration
BACKUP_SOURCE="/var/www /etc /home"
BACKUP_DEST="/backup"
BACKUP_PREFIX="backup"
RETENTION_DAYS=7
COMPRESSION="gzip" # gzip, bzip2, xz

# Create backup filename with timestamp
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="${BACKUP_PREFIX}_${TIMESTAMP}"

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a /var/log/backup.log
}

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    log "ERROR: This script must be run as root"
    exit 1
fi

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DEST"

# Function to perform backup
perform_backup() {
    local source_dir="$1"
    local backup_file="${BACKUP_DEST}/${BACKUP_NAME}_$(basename $source_dir).tar"
    
    log "Backing up $source_dir..."
    
    # Create tar archive
    if tar -cf "$backup_file" "$source_dir" 2>/dev/null; then
        # Compress based on selected method
        case "$COMPRESSION" in
            gzip)
                gzip "$backup_file"
                backup_file="${backup_file}.gz"
                ;;
            bzip2)
                bzip2 "$backup_file"
                backup_file="${backup_file}.bz2"
                ;;
            xz)
                xz "$backup_file"
                backup_file="${backup_file}.xz"
                ;;
        esac
        
        log "Successfully backed up $source_dir to $backup_file"
        return 0
    else
        log "ERROR: Failed to backup $source_dir"
        return 1
    fi
}

# Function to rotate old backups
rotate_backups() {
    log "Rotating old backups..."
    
    # Find and delete backups older than retention period
    find "$BACKUP_DEST" -name "${BACKUP_PREFIX}_*" -type f -mtime +$RETENTION_DAYS -exec rm {} \; -print | while read file; do
        log "Deleted old backup: $file"
    done
}

# Main backup process
log "Starting backup process"

# Perform backups
backup_count=0
error_count=0

for source in $BACKUP_SOURCE; do
    if [ -e "$source" ]; then
        if perform_backup "$source"; then
            ((backup_count++))
        else
            ((error_count++))
        fi
    else
        log "WARNING: Source $source does not exist, skipping"
    fi
done

# Rotate old backups
rotate_backups

# Summary
log "Backup process completed: $backup_count successful, $error_count failed"

# Send email notification if configured
if [ -n "$BACKUP_EMAIL" ]; then
    echo "Backup completed on $(hostname) at $(date)" | mail -s "Backup Report" "$BACKUP_EMAIL"
fi

exit $error_count