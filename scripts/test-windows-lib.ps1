param(
    [string]$TestName = "mcp_server::tests::razedloops_diagnostic_bundle_is_advertised_read_only"
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$deps = Join-Path $projectRoot "src-tauri\target\debug\deps"
$manifest = Join-Path $projectRoot "src-tauri\tests\windows-test.manifest"

$exe = Get-ChildItem $deps -Filter "repotunnel_lib-*.exe" -File |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (-not $exe) { throw "No compiled library test executable found. Run cargo test --lib --no-run first." }
if (-not (Test-Path $manifest)) { throw "Test manifest missing: $manifest" }

$kitRoot = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"
$mt = Get-ChildItem $kitRoot -Filter "mt.exe" -File -Recurse |
    Where-Object { $_.FullName -match '\\x64\\mt\.exe$' } |
    Sort-Object FullName -Descending | Select-Object -First 1
if (-not $mt) { throw "Microsoft mt.exe (x64) not found in Windows Kits." }

$work = Join-Path $env:TEMP "RazedTunnel-Manifest-Test"
New-Item -ItemType Directory -Path $work -Force | Out-Null
$copy = Join-Path $work $exe.Name
Copy-Item $exe.FullName $copy -Force

& $mt.FullName "-manifest" $manifest "-outputresource:${copy};#1"
if ($LASTEXITCODE -ne 0) { throw "Failed to embed Windows Common Controls v6 manifest." }

& $copy $TestName --exact --nocapture
exit $LASTEXITCODE
