#!/usr/bin/env bash
set -euo pipefail

# install-scheduler.sh — Install RepublicOS Skill Engine as a daily cron job
# Usage: ./install-scheduler.sh [options]
#   -t, --time     Daily run time in HH:MM format (default: 03:00)
#   -p, --python   Path to Python executable (default: auto-detect)
#   -s, --stages   Comma-separated stages to run (default: all)
#   -f, --full     Force full rediscovery on each run
#   -r, --remove   Remove the cron job instead of installing

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCHEDULER_SCRIPT="$SCRIPT_DIR/scheduler.py"
TIME="${TIME:-03:00}"
PYTHON=""
STAGES=""
FULL=false
REMOVE=false

usage() {
    echo "Usage: $0 [options]"
    echo "  -t, --time HH:MM   Daily run time (default: 03:00)"
    echo "  -p, --python PATH  Python executable path"
    echo "  -s, --stages LIST  Comma-separated stages (discovery,analyzer,...)"
    echo "  -f, --full         Force full rediscovery"
    echo "  -r, --remove       Remove the cron job"
    exit 1
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -t|--time) TIME="$2"; shift 2 ;;
        -p|--python) PYTHON="$2"; shift 2 ;;
        -s|--stages) STAGES="$2"; shift 2 ;;
        -f|--full) FULL=true; shift ;;
        -r|--remove) REMOVE=true; shift ;;
        *) usage ;;
    esac
done

CRON_LABEL="# RepublicOS-SkillEngine"

if [ "$REMOVE" = true ]; then
    echo "Removing cron job..."
    crontab -l 2>/dev/null | grep -v "$CRON_LABEL" | grep -v "$SCHEDULER_SCRIPT" | crontab -
    echo "Cron job removed."
    exit 0
fi

if [ -z "$PYTHON" ]; then
    PYTHON="$(command -v python3 || command -v python || true)"
fi

if [ -z "$PYTHON" ] || [ ! -x "$PYTHON" ]; then
    echo "Error: Python not found. Specify with -p or --python."
    exit 1
fi

if [ ! -f "$SCHEDULER_SCRIPT" ]; then
    echo "Error: scheduler.py not found at $SCHEDULER_SCRIPT"
    exit 1
fi

# Parse time
HOUR="${TIME%%:*}"
MINUTE="${TIME##*:}"
# Remove leading zeros for arithmetic
HOUR="$((10#$HOUR))"
MINUTE="$((10#$MINUTE))"

ARGS="run"
if [ -n "$STAGES" ]; then
    ARGS="$ARGS --stages $(echo "$STAGES" | tr ',' ' ')"
fi
if [ "$FULL" = true ]; then
    ARGS="$ARGS --full"
fi

CRON_LINE="$MINUTE $HOUR * * * cd $SCRIPT_DIR && $PYTHON $SCHEDULER_SCRIPT $ARGS >> $SCRIPT_DIR/registry/scheduler/scheduler-cron.log 2>&1"

# Backup existing crontab
(crontab -l 2>/dev/null || true) > /tmp/cron_backup_$$.txt

# Remove any existing entry for this script
grep -v "$CRON_LABEL" /tmp/cron_backup_$$.txt | grep -v "$SCHEDULER_SCRIPT" > /tmp/cron_new_$$.txt || true

# Add new entry
echo "$CRON_LINE $CRON_LABEL" >> /tmp/cron_new_$$.txt

crontab /tmp/cron_new_$$.txt

rm -f /tmp/cron_backup_$$.txt /tmp/cron_new_$$.txt

echo "Cron job installed successfully."
echo "  Python: $PYTHON"
echo "  Script: $SCHEDULER_SCRIPT"
echo "  Time:   $TIME daily"
echo "  Stages: ${STAGES:-all}"
if [ "$FULL" = true ]; then
    echo "  Mode:   Full rediscovery"
fi
echo ""
echo "To run manually:"
echo "  $PYTHON $SCHEDULER_SCRIPT run"
echo ""
echo "To remove:"
echo "  $0 --remove"
