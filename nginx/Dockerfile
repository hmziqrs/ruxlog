FROM nginx:1.27.2-alpine

# Copy your template nginx configuration file
COPY nginx.conf.template /etc/nginx/templates/nginx.conf.template

# Remove default nginx configuration file
RUN rm -rf /etc/nginx/conf.d/default.conf

# # Set environment variables with default values
# ENV PROJECT=rux_local

# Use the entrypoint script to substitute environment variables
CMD ["/docker-entrypoint.sh", "nginx", "-g", "daemon off;"]
