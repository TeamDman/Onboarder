#!/usr/bin/env pwsh
mkcert onboarder 127.0.0.1
# openssl req -x509 -newkey rsa:4096 -nodes -sha256 -keyout localhost.key -out localhost.crt -days 365 -config openssl.cnf
