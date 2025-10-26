#!/usr/bin/env node

/**
 * Vibe Kanban MCP HTTP Server
 *
 * This wraps the Vibe Kanban MCP server (which uses stdio transport)
 * with an HTTP transport layer, making it accessible via HTTP POST requests.
 *
 * This enables integration with MCP clients that require HTTP transport.
 */

import express from 'express';
import { spawn } from 'child_process';
import { StreamableHTTPServerTransport } from '@modelcontextprotocol/sdk/server/streamableHttp.js';

const app = express();
app.use(express.json());

const PORT = process.env.MCP_PORT || 9717;

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({ status: 'ok', service: 'vibe-kanban-mcp', port: PORT });
});

// MCP endpoint
app.post('/mcp', async (req, res) => {
  try {
    // Spawn the Vibe Kanban MCP process
    const mcpProcess = spawn('npx', ['-y', 'vibe-kanban@latest', '--mcp'], {
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let responseData = '';
    let errorData = '';

    // Handle stdout (MCP responses)
    mcpProcess.stdout.on('data', (data) => {
      responseData += data.toString();
    });

    // Handle stderr (errors)
    mcpProcess.stderr.on('data', (data) => {
      errorData += data.toString();
      console.error('MCP stderr:', data.toString());
    });

    // Send the request to the MCP process
    mcpProcess.stdin.write(JSON.stringify(req.body) + '\n');
    mcpProcess.stdin.end();

    // Wait for the process to complete
    mcpProcess.on('close', (code) => {
      if (code !== 0) {
        console.error(`MCP process exited with code ${code}`);
        console.error('Error output:', errorData);
        return res.status(500).json({
          error: 'MCP process failed',
          code: code,
          stderr: errorData
        });
      }

      try {
        // Parse and send the MCP response
        const response = JSON.parse(responseData);
        res.json(response);
      } catch (parseError) {
        console.error('Failed to parse MCP response:', parseError);
        console.error('Raw response:', responseData);
        res.status(500).json({
          error: 'Failed to parse MCP response',
          raw: responseData
        });
      }
    });

    // Handle process errors
    mcpProcess.on('error', (error) => {
      console.error('Failed to start MCP process:', error);
      res.status(500).json({
        error: 'Failed to start MCP process',
        message: error.message
      });
    });

  } catch (error) {
    console.error('MCP handler error:', error);
    res.status(500).json({
      error: 'Internal server error',
      message: error.message
    });
  }
});

// Start the server
app.listen(PORT, '0.0.0.0', () => {
  console.log(`Vibe Kanban MCP HTTP Server listening on http://0.0.0.0:${PORT}/mcp`);
  console.log(`Health check available at http://0.0.0.0:${PORT}/health`);
});

// Handle graceful shutdown
process.on('SIGTERM', () => {
  console.log('SIGTERM received, shutting down gracefully...');
  process.exit(0);
});

process.on('SIGINT', () => {
  console.log('SIGINT received, shutting down gracefully...');
  process.exit(0);
});
