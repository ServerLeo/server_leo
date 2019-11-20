# server_leo
Commands to generate keys and certificates:

openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'

openssl pkcs12 -export -out identity.pfx -inkey key.pem -in cert.pem

sudo mkdir /usr/local/share/ca-certificates/extra

sudo cp cert.pem /usr/local/share/ca-certificates/extra/cert.crt

sudo update-ca-certificates
