[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$InstallerPath,

    [string]$OutputDirectory = "release-artifacts"
)

$ErrorActionPreference = "Stop"

$installer = Get-Item -LiteralPath $InstallerPath
if (-not $installer -or $installer.PSIsContainer) {
    throw "Installer path does not point to a file."
}
if ($installer.Extension -ne ".exe" -or $installer.Name -notmatch "[_-]x64-setup\.exe$") {
    throw "Expected a Tauri x64 NSIS setup executable. Found '$($installer.Name)'."
}
if ($installer.Length -lt 1MB) {
    throw "Installer is unexpectedly small ($($installer.Length) bytes)."
}

$configPath = Join-Path $PSScriptRoot "../src-tauri/tauri.conf.json"
$config = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json
$expectedPrefix = "Survey Copilot_$($config.version)_x64-setup.exe"
if ($installer.Name -ne $expectedPrefix) {
    throw "Installer filename '$($installer.Name)' does not match configured product version '$($config.version)' (expected '$expectedPrefix')."
}
if ($config.bundle.windows.nsis.installMode -ne "currentUser") {
    throw "The installer is not configured for the approved currentUser install mode."
}

New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$destination = Join-Path (Resolve-Path $OutputDirectory).Path $installer.Name
Copy-Item -LiteralPath $installer.FullName -Destination $destination -Force

$hash = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant()
$signature = Get-AuthenticodeSignature -LiteralPath $destination
if ($signature.Status -notin @("Valid", "NotSigned")) {
    throw "Installer Authenticode signature state is unexpected: $($signature.Status)."
}
if ($signature.Status -eq "NotSigned") {
    Write-Warning "Installer is unsigned. This is an internal CI artifact only and must not be published as a public release."
}

$hashLine = "$hash  $($installer.Name)"
$hashFile = "$destination.sha256"
Set-Content -LiteralPath $hashFile -Value $hashLine -Encoding utf8
Set-Content -LiteralPath (Join-Path $OutputDirectory "SHA256SUMS.txt") -Value $hashLine -Encoding utf8

$manifest = [ordered]@{
    productName = [string]$config.productName
    version = [string]$config.version
    architecture = "x86_64"
    packageFormat = "NSIS"
    installMode = [string]$config.bundle.windows.nsis.installMode
    installerFile = $installer.Name
    sizeBytes = [int64](Get-Item -LiteralPath $destination).Length
    sha256 = $hash
    authenticodeStatus = [string]$signature.Status
    authenticodeSigner = if ($signature.SignerCertificate) { $signature.SignerCertificate.Subject } else { $null }
    appDataDirectory = "%APPDATA%\com.surveycopilot.desktop"
    appDataUninstallDefault = "Preserved unless the user selects the NSIS Delete app data checkbox"
    credentialStoreUninstallDefault = "Windows Credential Manager secrets are not removed by the NSIS app-data directory cleanup"
    sourceCommit = if ($env:GITHUB_SHA) { $env:GITHUB_SHA } else { $null }
    workflowRun = if ($env:GITHUB_RUN_ID) { $env:GITHUB_RUN_ID } else { $null }
    generatedAtUtc = [DateTime]::UtcNow.ToString("o")
}
$manifestPath = Join-Path $OutputDirectory "build-manifest.json"
$manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding utf8

Write-Host "Installer: $destination"
Write-Host "SHA-256: $hash"
Write-Host "Authenticode: $($signature.Status)"
