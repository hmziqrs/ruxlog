FROM nginx:1.27.2-alpine

# Copy your custom nginx configuration file
COPY nginx.conf /etc/nginx/nginx.conf

# Remove default nginx configuration file
RUN rm -rf /etc/nginx/conf.d/default.conf
