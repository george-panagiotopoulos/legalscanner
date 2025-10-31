# Build stage
FROM node:20-alpine as builder

WORKDIR /app

# Accept build argument for API URL
ARG VITE_API_BASE_URL=http://localhost:5301

# Copy package files
COPY legalscanner-ui/package*.json ./

# Install dependencies
RUN npm install

# Copy source code
COPY legalscanner-ui/ ./

# Build for production (environment variable available during build)
ENV VITE_API_BASE_URL=${VITE_API_BASE_URL}
RUN npm run build

# Runtime stage
FROM nginx:alpine

# Copy built assets from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf

# Expose port
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
