Raw dogging the metal:

Install postgresql, secure it, and create a user and database for the app.

testing
rm test-setup.sh && nano test-setup.sh && chmod +x test-setup.sh && ./test-setup.sh

chmod +x test-setup.sh && ./test-setup.sh

sudo certbot certonly --manual --manual-auth-hook /etc/letsencrypt/acme-dns-auth.py --preferred-challenges dns --debug-challenges -d \*.your-domain -d your-domain

sudo certbot certonly --manual --manual-auth-hook /etc/letsencrypt/acme-dns-auth.py --preferred-challenges dns --debug-challenges -d \*.hmziq.rs -d hmziq.rs

Certificate is saved at: /etc/letsencrypt/live/hmziq.rs/fullchain.pem
Key is saved at: /etc/letsencrypt/live/hmziq.rs/privkey.pem
