FROM nginx:1.27.2-alpine

# Copy your template nginx configuration file
COPY nginx.conf.template /etc/nginx/nginx.conf.template

# Remove default nginx configuration file
RUN rm -rf /etc/nginx/conf.d/default.conf

# Copy the custom entrypoint script
COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

# Set the entrypoint to your custom script
ENTRYPOINT ["/docker-entrypoint.sh"]
