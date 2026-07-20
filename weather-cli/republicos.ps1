param(
    [string]$Command = "doctor"
)

function Doctor {
    $root = Split-Path -Parent $PSCommandPath
    $errors = @()
    $warnings = @()

    Write-Host "`nRepublicOS Status`n" -ForegroundColor Cyan

    # Required directories
    $dirs = @(
        ".constitution", ".organization", ".protocols",
        ".memory", ".gates"
    )

    foreach ($dir in $dirs) {
        $path = Join-Path $root $dir
        if (Test-Path $path) {
            $count = @(Get-ChildItem -LiteralPath $path -File).Count
            Write-Host ("  [OK] " + $dir + " (" + $count + " files)") -ForegroundColor Green
        } else {
            Write-Host ("  [XX] " + $dir + " (missing)") -ForegroundColor Red
            $errors += "Missing directory: " + $dir
        }
    }

    # Required files
    $files = @(
        "AI_ENTRYPOINT.md"
    )

    foreach ($file in $files) {
        $path = Join-Path $root $file
        if (Test-Path $path) {
            Write-Host ("  [OK] " + $file) -ForegroundColor Green
        } else {
            Write-Host ("  [XX] " + $file + " (missing)") -ForegroundColor Red
            $errors += "Missing file: " + $file
        }
    }

    # Check AI_ENTRYPOINT.md content
    $entryPoint = Join-Path $root "AI_ENTRYPOINT.md"
    if (Test-Path $entryPoint) {
        $content = Get-Content $entryPoint -Raw
        if ($content -notmatch "CONSTITUTION") {
            $warnings += "AI_ENTRYPOINT.md may not reference CONSTITUTION.md"
        }
    }

    # Check MISSION.md is not placeholder
    $mission = Join-Path $root ".constitution\MISSION.md"
    if (Test-Path $mission) {
        $content = Get-Content $mission -Raw
        if ($content -match "Define your mission here" -or $content.Trim() -eq "") {
            $warnings += "MISSION.md still contains placeholder text"
        }
    }

    # Check SUCCESS.md has criteria
    $success = Join-Path $root ".constitution\SUCCESS.md"
    if (Test-Path $success) {
        $content = Get-Content $success -Raw
        if ($content -notmatch "- \[ \]") {
            $warnings += "SUCCESS.md has no unchecked criteria"
        }
    }

    # Summary
    Write-Host ""
    if ($errors.Count -eq 0 -and $warnings.Count -eq 0) {
        Write-Host "  Ready for AI" -ForegroundColor Green
        Write-Host "  Hand this repository to any AI and say:" -ForegroundColor Gray
        Write-Host "  Analyze this repository. Read AI_ENTRYPOINT.md. Behave according to RepublicOS." -ForegroundColor Yellow
        return 0
    } else {
        if ($errors.Count -gt 0) {
            Write-Host "  Errors:" -ForegroundColor Red
            foreach ($e in $errors) { Write-Host ("    [XX] " + $e) -ForegroundColor Red }
        }
        if ($warnings.Count -gt 0) {
            Write-Host "  Warnings:" -ForegroundColor Yellow
            foreach ($w in $warnings) { Write-Host ("    [!] " + $w) -ForegroundColor Yellow }
        }
        Write-Host "  Not ready for AI" -ForegroundColor Red
        return 1
    }
}

function Help {
    Write-Host "RepublicOS CLI" -ForegroundColor Cyan
    Write-Host "  doctor     Validate RepublicOS setup"
    Write-Host "  help       Show this message"
}

switch ($Command.ToLower()) {
    "doctor" { exit (Doctor) }
    default  { Help }
}
