#!/bin/bash
# RepublicOS Governor — shell wrapper
# Delegates to the Python core.
# Usage: ./governor.sh <command> [args...]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
python "$SCRIPT_DIR/.governor/governor.py" "$@"
