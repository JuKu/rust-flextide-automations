-- Add Docs module settings group and settings
-- Supports both MySQL and PostgreSQL
--
-- This migration adds:
-- 1. Settings group "module_docs" with title "Docs"
-- 2. Setting "module_docs_page_summary_ai_provider" - dropdown to select AI provider
-- 3. Setting "module_docs_openai_api_key" - textfield for OpenAI API key
-- 4. Setting "module_docs_openai_model" - dropdown for OpenAI model selection

-- ============================================================================
-- INSERT SETTINGS GROUP
-- ============================================================================

INSERT INTO organizational_settings_groups (unique_name, title, description, created_at)
SELECT 'module_docs', 'Docs', 'Documentation module settings', CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM organizational_settings_groups WHERE unique_name = 'module_docs');

-- ============================================================================
-- INSERT SETTINGS
-- ============================================================================

-- AI Provider setting (dropdown)
INSERT INTO organizational_settings (
    name,
    organizational_settings_group_name,
    title,
    description,
    type,
    metadata,
    created_at,
    updated_at
)
SELECT 
    'module_docs_page_summary_ai_provider',
    'module_docs',
    'Page Summary AI Provider',
    'Select the AI provider to use for generating page summaries',
    'dropdown',
    '{"options": [{"value": "openai", "label": "OpenAI"}, {"value": "claude", "label": "Claude (Anthropic)"}, {"value": "gemini", "label": "Gemini (Google)"}]}',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM organizational_settings WHERE name = 'module_docs_page_summary_ai_provider');

-- OpenAI API Key setting (textfield)
INSERT INTO organizational_settings (
    name,
    organizational_settings_group_name,
    title,
    description,
    type,
    metadata,
    created_at,
    updated_at
)
SELECT 
    'module_docs_openai_api_key',
    'module_docs',
    'OpenAI API Key',
    'API key for OpenAI (required if OpenAI is selected as AI provider)',
    'textfield',
    '{"placeholder": "sk-...", "required": false}',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM organizational_settings WHERE name = 'module_docs_openai_api_key');

-- OpenAI Model setting (dropdown)
INSERT INTO organizational_settings (
    name,
    organizational_settings_group_name,
    title,
    description,
    type,
    metadata,
    created_at,
    updated_at
)
SELECT 
    'module_docs_openai_model',
    'module_docs',
    'OpenAI Model',
    'OpenAI model to use for page summaries',
    'dropdown',
    '{"options": [{"value": "gpt-4o-mini", "label": "GPT-4o Mini (Fast & Cost-effective)"}, {"value": "gpt-4o", "label": "GPT-4o (High Quality)"}, {"value": "gpt-4-turbo", "label": "GPT-4 Turbo"}, {"value": "gpt-3.5-turbo", "label": "GPT-3.5 Turbo"}]}',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM organizational_settings WHERE name = 'module_docs_openai_model');

