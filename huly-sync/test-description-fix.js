#!/usr/bin/env node

/**
 * Test script to verify full description extraction from Huly issues
 */

import fetch from 'node-fetch';

const HULY_MCP_URL = 'http://192.168.50.90:3457/mcp';

// Extract full description from Huly issue detail response
function extractFullDescription(detailText) {
  const lines = detailText.split('\n');
  let inDescription = false;
  let description = [];

  for (const line of lines) {
    // Start capturing after ## Description header
    if (line.trim() === '## Description') {
      inDescription = true;
      continue;
    }

    // Stop at the next ## header (like ## Recent Comments)
    if (inDescription && line.trim().startsWith('## ')) {
      break;
    }

    // Capture description lines
    if (inDescription) {
      description.push(line);
    }
  }

  // Join and trim the description
  return description.join('\n').trim();
}

// Simple MCP client
class MCPClient {
  constructor(url, name) {
    this.url = url;
    this.name = name;
    this.requestId = 1;
    this.sessionId = null;
  }

  async initialize() {
    console.log(`[${this.name}] Initializing...`);

    const initResult = await this.call('initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: { name: 'test-client', version: '1.0.0' },
    });

    console.log(`[${this.name}] ✓ Initialized`);
    return initResult;
  }

  async call(method, params = {}) {
    const headers = {
      'Content-Type': 'application/json',
      'Accept': 'application/json, text/event-stream',
    };

    if (this.sessionId) {
      headers['mcp-session-id'] = this.sessionId;
    }

    const response = await fetch(this.url, {
      method: 'POST',
      headers,
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: this.requestId++,
        method,
        params,
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();

    if (data.error) {
      throw new Error(`MCP Error: ${data.error.message}`);
    }

    // Store session ID from response headers
    const sessionHeader = response.headers.get('mcp-session-id');
    if (sessionHeader && !this.sessionId) {
      this.sessionId = sessionHeader;
      console.log(`[${this.name}] Session ID: ${this.sessionId}`);
    }

    return data.result;
  }

  async callTool(toolName, params) {
    return this.call('tools/call', {
      name: toolName,
      arguments: params,
    });
  }
}

async function testDescriptionExtraction() {
  console.log('Testing Full Description Extraction for LMS-29\n');

  const client = new MCPClient(HULY_MCP_URL, 'Huly');
  await client.initialize();

  console.log('\n[Test] Fetching LMS-29 details...');
  const result = await client.callTool('huly_query', {
    entity_type: 'issue',
    mode: 'get',
    issue_identifier: 'LMS-29',
  });

  const resultText = typeof result === 'string' ? result : result.toString();

  console.log('\n[Test] Raw response length:', resultText.length, 'characters');
  console.log('[Test] First 200 chars:', resultText.substring(0, 200));

  const fullDescription = extractFullDescription(resultText);

  console.log('\n[Test] Extracted description length:', fullDescription.length, 'characters');
  console.log('[Test] Description preview (first 500 chars):');
  console.log('---');
  console.log(fullDescription.substring(0, 500));
  console.log('---');

  console.log('\n[Test] Description includes "Summary" section?', fullDescription.includes('## Summary'));
  console.log('[Test] Description includes "Completed Optimizations"?', fullDescription.includes('## Completed Optimizations'));
  console.log('[Test] Description includes "High Priority Issues"?', fullDescription.includes('## High Priority Issues'));

  if (fullDescription.length > 100 && fullDescription.includes('## Summary')) {
    console.log('\n✅ SUCCESS: Full multi-line description extracted correctly!');
  } else {
    console.log('\n❌ FAILED: Description extraction incomplete');
  }
}

testDescriptionExtraction().catch(console.error);
