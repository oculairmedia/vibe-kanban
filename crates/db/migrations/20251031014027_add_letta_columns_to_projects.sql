-- Add Letta agent tracking columns to projects table
-- These columns track the state of Letta agents managing each project

-- Letta agent ID associated with this project
ALTER TABLE projects ADD COLUMN letta_agent_id TEXT;

-- Letta folder ID for project documentation and context
ALTER TABLE projects ADD COLUMN letta_folder_id TEXT;

-- Letta source ID for project knowledge base
ALTER TABLE projects ADD COLUMN letta_source_id TEXT;

-- Timestamp of last synchronization with Letta agent (Unix timestamp in milliseconds)
ALTER TABLE projects ADD COLUMN letta_last_sync_at INTEGER;
