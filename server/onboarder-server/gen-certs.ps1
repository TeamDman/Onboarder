#!/usr/bin/env pwsh
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$caKey = "onboarder-ca.key"
$caCert = "onboarder-ca.crt"

if (-not (Test-Path $caKey) -or -not (Test-Path $caCert)) {
	throw "Missing $caKey or $caCert. Run .\gen-ca.ps1 first."
}

$serverKey = "localhost.key"
$serverCsr = "localhost.csr"
$serverCert = "localhost.crt"

openssl req -new -newkey rsa:4096 -nodes -sha256 `
	-keyout $serverKey `
	-out $serverCsr `
	-config openssl.cnf

openssl x509 -req -in $serverCsr `
	-CA $caCert `
	-CAkey $caKey `
	-CAcreateserial `
	-out $serverCert `
	-days 365 `
	-sha256 `
	-extensions v3_req `
	-extfile openssl.cnf

Remove-Item $serverCsr -ErrorAction SilentlyContinue
Write-Host "Generated $serverCert signed by Onboarder Dev CA."
