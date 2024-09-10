if [ ! -d "../certificates" ]; then
  echo "Creating directory $CERT_DIR..."
  mkdir -p "../certificates"
fi

openssl req -x509 -newkey rsa:2048 -keyout "../certificates/key.pem" -out "../certificates/cert.pem" -days 365 -nodes

echo "Certificates have been generated and stored in ../certificates"

