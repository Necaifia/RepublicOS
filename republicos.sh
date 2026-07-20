#!/bin/bash
# RepublicOS doctor — validate setup
# Usage: ./republicos.sh doctor

doctor() {
    local root="$(cd "$(dirname "$0")" && pwd)"
    local errors=()
    local warnings=()

    echo ""
    echo "RepublicOS Status"
    echo ""

    # Required directories
    for dir in .constitution .organization .protocols .memory .gates; do
        if [ -d "$root/$dir" ]; then
            count=$(find "$root/$dir" -maxdepth 1 -type f | wc -l)
            echo "  [OK] $dir ($count files)"
        else
            echo "  [XX] $dir (missing)"
            errors+=("Missing directory: $dir")
        fi
    done

    # Required files
    if [ -f "$root/AI_ENTRYPOINT.md" ]; then
        echo "  [OK] AI_ENTRYPOINT.md"
    else
        echo "  [XX] AI_ENTRYPOINT.md (missing)"
        errors+=("Missing file: AI_ENTRYPOINT.md")
    fi

    # Check MISSION.md
    if [ -f "$root/.constitution/MISSION.md" ]; then
        if grep -q "Define your mission here" "$root/.constitution/MISSION.md"; then
            warnings+=("MISSION.md still contains placeholder text")
        fi
    fi

    # Check SUCCESS.md
    if [ -f "$root/.constitution/SUCCESS.md" ]; then
        if ! grep -q "\[ \]" "$root/.constitution/SUCCESS.md" 2>/dev/null; then
            warnings+=("SUCCESS.md has no unchecked criteria")
        fi
    fi

    echo ""
    if [ ${#errors[@]} -eq 0 ] && [ ${#warnings[@]} -eq 0 ]; then
        echo "  Ready for AI"
        echo "  Hand this repository to any AI and say:"
        echo "  Analyze this repository. Read AI_ENTRYPOINT.md. Behave according to RepublicOS."
        return 0
    else
        for e in "${errors[@]}"; do echo "  [XX] $e"; done
        for w in "${warnings[@]}"; do echo "  [!] $w"; done
        echo "  Not ready for AI"
        return 1
    fi
}

case "${1:-doctor}" in
    doctor) doctor ;;
    help|--help|-h)
        echo "RepublicOS CLI"
        echo "  doctor     Validate RepublicOS setup"
        echo "  help       Show this message"
        ;;
esac
