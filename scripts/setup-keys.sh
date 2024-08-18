mkdir -p ../keys

openssl genpkey -algorithm RSA -out ../keys/private.pem

openssl rsa -pubout -in ../keys/private.pem -out ../keys/public.pem

echo "Keys have been generated and saved to ../keys/private.pem and ../keys/public.pem"