#!/usr/bin/env pwsh
Set-StrictMode -Version Latest

function Invoke-Luna {
    param(
        [Parameter(Mandatory)]
        [string]$Filename
    )

    $exe = Join-Path "." "target\debug\luna-rs.exe"
    if (-not (Test-Path $exe)) {
        $exe = Join-Path "." "target\debug\luna-rs"
    }

    # Native tools writing to stderr can trigger NativeCommandError when ErrorActionPreference=Stop.
    # Temporarily relax error handling so we can use exit codes as the truth.
    $oldEap = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        & $exe $Filename *> $null
        return ($LASTEXITCODE -eq 0)
    }
    finally {
        $ErrorActionPreference = $oldEap
    }
}

# Fail fast for cargo only
$ErrorActionPreference = 'Stop'
cargo test
cargo build
$ErrorActionPreference = 'Continue'

Write-Host "=== Testing Success Cases ==="
$successCount = 0
$totalSuccess = 0

Get-ChildItem -Path "tests/success" -File -ErrorAction SilentlyContinue | ForEach-Object {
    $filename = $_.FullName
    $totalSuccess++

    Write-Host ("Running success test: {0}" -f $filename)
    if (Invoke-Luna -Filename $filename) {
        Write-Host ("PASSED: {0}" -f $filename)
        $successCount++
    } else {
        Write-Host ("FAILED: {0} (expected to pass)" -f $filename)
    }
    Write-Host ""
}

Write-Host ("Success Tests: {0}/{1} passed" -f $successCount, $totalSuccess)
Write-Host ""

Write-Host "=== Testing Failure Cases ==="
$failureCount = 0
$totalFailure = 0

Get-ChildItem -Path "tests/failure" -File -ErrorAction SilentlyContinue | ForEach-Object {
    $filename = $_.FullName
    $totalFailure++

    Write-Host ("Running failure test: {0}" -f $filename)
    if (-not (Invoke-Luna -Filename $filename)) {
        Write-Host ("PASSED: {0} (correctly failed)" -f $filename)
        $failureCount++
    } else {
        Write-Host ("FAILED: {0} (should have failed but passed)" -f $filename)
    }
    Write-Host ""
}

Write-Host ("Failure Tests: {0}/{1} passed" -f $failureCount, $totalFailure)
Write-Host ""

$totalTests  = $totalSuccess + $totalFailure
$totalPassed = $successCount + $failureCount

Write-Host "=== Test Summary ==="
Write-Host ("Success Tests: {0}/{1} passed" -f $successCount, $totalSuccess)
Write-Host ("Failure Tests: {0}/{1} passed" -f $failureCount, $totalFailure)
Write-Host ("Total: {0}/{1} tests passed" -f $totalPassed, $totalTests)

if ($totalPassed -eq $totalTests) {
    Write-Host "All tests passed!"
    exit 0
} else {
    Write-Host "Some tests failed!"
    exit 1
}