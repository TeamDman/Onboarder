#!/usr/bin/env pwsh
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$caKey = "onboarder-ca.key"
$caCert = "onboarder-ca.crt"

if ((Test-Path $caKey) -or (Test-Path $caCert)) {
    throw "CA material already exists. Remove $caKey and $caCert if you want to recreate them."
}

$subject = "/C=CA/ST=Ontario/O=TeamDman/CN=Onboarder Dev CA"

openssl req -x509 -newkey rsa:4096 -nodes -sha256 `
    -keyout $caKey `
    -out $caCert `
    -days 3650 `
    -subj $subject

Import-Certificate -FilePath (Resolve-Path $caCert) -CertStoreLocation Cert:\CurrentUser\Root | Out-Null
Write-Host "Trusted Onboarder Dev CA in CurrentUser\\Root."