# Stage 1: Build
FROM node:20-alpine AS builder

WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm ci --only=production

# Stage 2: Runtime
FROM node:20-alpine

RUN apk add --no-cache chrony openssl

WORKDIR /app

# Copy node_modules from builder
COPY --from=builder /app/node_modules ./node_modules

# Copy app
COPY server/ ./server/
COPY public/ ./public/
COPY .env.example ./.env.example

# Create certs directory
RUN mkdir -p /app/certs

# Expose ports
EXPOSE 13013
EXPOSE 13014

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD wget -qO- http://localhost:13013/api/status || exit 1

# Start
CMD ["node", "server/index.js"]
