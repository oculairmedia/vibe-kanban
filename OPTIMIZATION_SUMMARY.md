# vibe-kanban-mcp Memory Leak Fix & Optimization

## Date: 2025-10-27

## Problem Summary
The `vibe-kanban-mcp` container experienced a catastrophic memory leak, consuming **49.86GB** out of 58.59GB (85% of available memory), nearly crashing the system.

## Root Cause
**Child Process Accumulation in Supergateway Stateless Mode**

1. **Uncontrolled Process Spawning**: Each MCP connection spawned new child processes without proper lifecycle management
2. **No Session Cleanup**: Stateless mode had no timeout mechanism to kill idle sessions
3. **Binary Extraction Conflicts**: Concurrent process spawns caused `ETXTBSY` file locking errors
4. **No Memory Limits**: Container could consume unlimited memory
5. **Zombie Processes**: No init system to reap orphaned processes

## Evidence
- **Before**: 49.86GB memory usage, 10+ concurrent processes per session
- **Process Pattern**: Multiple `npm exec vibe-kanban --mcp` and `vibe-kanban-mcp` binaries accumulating
- **Logs**: Repeated initialization failures, file locking errors, request timeouts

## Optimizations Implemented

### 1. Memory Limits (docker-compose.yml:24-26)
```yaml
mem_limit: 2g              # Hard cap at 2GB
memswap_limit: 2g          # Prevent swap usage
mem_reservation: 512m      # Guaranteed minimum
```

### 2. Node.js Heap Limit (docker-compose.yml:20-21)
```yaml
environment:
  - NODE_OPTIONS=--max-old-space-size=1536  # 1.5GB heap limit
```

### 3. Supergateway Session Management (docker-compose.yml:13)
```bash
--sessionTimeout 300000           # Kill idle sessions after 5 minutes
--maxConcurrentSessions 10        # Limit to 10 parallel connections
```

### 4. Process Lifecycle Management (docker-compose.yml:28)
```yaml
init: true  # Use tini to reap zombie processes
```

### 5. Persistent npm Cache (docker-compose.yml:18)
```yaml
volumes:
  - vibe-kanban-npm-cache:/usr/local/lib/node_modules  # Prevent extraction conflicts
```

## Results

### Memory Usage
- **Before**: 49.86GB / 58.59GB (85%)
- **After**: 76MB / 2GB (3.7%)
- **Reduction**: **99.8% memory savings**

### Process Count
- **Before**: 10+ processes accumulating per session
- **After**: 3 processes (stable)

### System Stability
- **Before**: 978MB available (critical)
- **After**: 52GB available (healthy)

### Memory Limits Applied
- Hard limit: 2,147,483,648 bytes (2GB)
- Swap limit: 2,147,483,648 bytes (2GB)
- Reservation: 536,870,912 bytes (512MB)

## Monitoring Recommendations

### Check Memory Usage
```bash
docker stats vibe-kanban-mcp --no-stream
```

### Check Process Count
```bash
docker exec vibe-kanban-mcp ps aux | grep vibe-kanban | wc -l
```

### Monitor Logs for Session Cleanup
```bash
docker logs vibe-kanban-mcp -f | grep -i "session"
```

### Alert Thresholds
- Memory > 1.5GB: Warning
- Memory > 1.8GB: Critical
- Process count > 15: Investigate
- Session duration > 10 minutes: Check timeout

## Testing Performed
✅ Container starts successfully with new limits
✅ Memory capped at 2GB (verified via docker inspect)
✅ Health endpoint responding (http://localhost:9717/health)
✅ MCP endpoint accessible
✅ Process count stable at 3 processes
✅ System memory recovered to 52GB available

## Files Modified
- `/opt/stacks/vibe-kanban/docker-compose.yml` - Main configuration
- `/opt/stacks/vibe-kanban/docker-compose.yml.backup` - Backup created

## Future Improvements
1. **Monitoring Dashboard**: Add Prometheus/Grafana to track memory over time
2. **Auto-Restart**: Configure alerts to auto-restart if memory > 1.8GB
3. **Connection Pooling**: Implement client-side connection pooling to reduce spawns
4. **Upgrade Supergateway**: Monitor for upstream fixes to stateless mode
5. **Log Aggregation**: Send logs to centralized system for pattern analysis

## Rollback Instructions
If issues occur:
```bash
cd /opt/stacks/vibe-kanban
cp docker-compose.yml.backup docker-compose.yml
docker-compose down
docker-compose up -d
```

## Contact
For questions about this fix, check:
- GitHub Issues: https://github.com/supercorp-ai/supergateway/issues
- Vibe Kanban Docs: /opt/stacks/vibe-kanban/CLAUDE.md
