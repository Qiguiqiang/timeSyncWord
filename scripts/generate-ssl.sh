#!/bin/bash
# Generate self-signed SSL certificates for OpenTimeSync
# Usage: ./scripts/generate-ssl.sh

CERT_DIR="$(dirname "$0")/../certs"
mkdir -p "$CERT_DIR"

DAYS=3650
KEY="$CERT_DIR/server.key"
CERT="$CERT_DIR/server.crt"

echo "Generating self-signed SSL certificates..."
echo "  Key:  $KEY"
echo "  Cert: $CERT"
echo "  Days: $DAYS"
echo ""

openssl req -x509 -nodes -newkey rsa:4096 \
  -keyout "$KEY" \
  -out "$CERT" \
  -days "$DAYS" \
  -subj "/C=CN/ST=State/L=City/O=OpenTimeSync/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,DNS:*.local,IP:127.0.0.1"

if [ $? -eq 0 ]; then
  echo ""
  echo "Certificates generated successfully!"
  echo ""
  echo "To enable HTTPS, set in your environment:"
  echo "  SSL_ENABLED=true"
  echo "  SSL_KEY_PATH=$KEY"
  echo "  SSL_CERT_PATH=$CERT"
  echo ""
  echo "Or add to .env file:"
  echo "  SSL_ENABLED=true"
  echo "  SSL_KEY_PATH=certs/server.key"
  echo "  SSL_CERT_PATH=certs/server.crt"
  echo ""
  echo "WARNING: Self-signed certificates will show a browser warning."
  echo "For production, use Let's Encrypt (certbot) instead."
else
  echo "Failed to generate certificates."
  echo "Make sure openssl is installed."
  exit 1
fi
