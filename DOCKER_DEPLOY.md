# Vibe Kanban - Docker Deployment Guide

This guide explains how to deploy Vibe Kanban using Docker Compose with all necessary CLI tools included.

## What's Included

The Docker image includes:
- **Vibe Kanban** - The main application
- **Claude Code** - Anthropic's AI coding assistant
- **OpenCode** - Additional development tools
- **Standard CLI tools**: Git, Vim, Nano, Bash, SSH client
- **Sudo support** for the app user

## Quick Start

### Using Docker Compose (Recommended)

```bash
# Clone the repository
git clone https://github.com/oculairmedia/vibe-kanban.git
cd vibe-kanban

# Start the services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop the services
docker-compose down
```

### Using Pre-built Image

```bash
# Pull the latest image
docker pull ghcr.io/oculairmedia/vibe-kanban:main

# Run the container
docker run -d \
  --name vibe-kanban \
  -p 3000:3000 \
  -v vibe-data:/data \
  -v vibe-config:/root/.config \
  ghcr.io/oculairmedia/vibe-kanban:main
```

## Accessing the Application

Once running, access Vibe Kanban at:
- **Web UI**: http://localhost:3000
- **Health Check**: http://localhost:3000/health

## Using CLI Tools Inside the Container

### Access the container shell

```bash
# Using docker-compose
docker-compose exec vibe-kanban bash

# Using docker directly
docker exec -it vibe-kanban bash
```

### Use Claude Code

```bash
# Inside the container
claude --version
claude
```

### Use OpenCode

```bash
# Inside the container
opencode --help
```

## Configuration

### Environment Variables

Edit `docker-compose.yml` to set environment variables:

```yaml
environment:
  - PORT=3000
  - NODE_ENV=production
  - ANTHROPIC_API_KEY=your_key_here  # For Claude Code
```

### Persistent Data

Data is stored in Docker volumes:
- `vibe-data` - Application data
- `vibe-config` - Configuration files

To backup:
```bash
docker run --rm \
  -v vibe-data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/vibe-data-backup.tar.gz /data
```

## Building from Source

```bash
# Build the image locally
docker-compose build

# Or build without compose
docker build -f Dockerfile.dev -t vibe-kanban:local .
```

## Troubleshooting

### Container won't start

Check logs:
```bash
docker-compose logs vibe-kanban
```

### Port already in use

Change the port mapping in `docker-compose.yml`:
```yaml
ports:
  - "3001:3000"  # Changed from 3000:3000
```

### Health check failing

Wait 40 seconds for the start period, then check:
```bash
docker-compose ps
docker exec vibe-kanban wget -O- http://localhost:3000/health
```

## Security Notes

- The application runs as non-root user `appuser` (UID 1001)
- Sudo is available for the app user if needed for CLI tools
- SSH client is included but no SSH server runs by default

## Development

To make changes and rebuild:

```bash
# Make your changes to the code
# Then rebuild and restart
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

## Support

For issues or questions:
- Original repo: https://github.com/BloopAI/vibe-kanban
- Fork with Docker setup: https://github.com/oculairmedia/vibe-kanban
