-- Migration: Create mutation_tests and mutation_results tables

CREATE TYPE mutation_test_status AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled');
CREATE TYPE test_result AS ENUM ('pending', 'killed', 'survived', 'timeout', 'error', 'skipped');

CREATE TABLE mutation_tests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    source_code TEXT NOT NULL,
    language TEXT NOT NULL,
    status mutation_test_status NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

CREATE TABLE mutation_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mutation_test_id UUID NOT NULL REFERENCES mutation_tests(id) ON DELETE CASCADE,
    mutation_type TEXT NOT NULL,
    original_code TEXT NOT NULL,
    mutated_code TEXT NOT NULL,
    line_number INT NOT NULL,
    column_number INT,
    test_result test_result NOT NULL,
    execution_time_ms BIGINT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);