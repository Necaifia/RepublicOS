@rem RepublicOS Governor — PowerShell wrapper
@rem Delegates to the Python core.
@rem Usage: .\governor.ps1 <command> [args...]

python "$PSScriptRoot\.governor\governor.py" @args
