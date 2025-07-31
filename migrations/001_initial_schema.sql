-- Create mutation_tests table
CREATE TABLE IF NOT EXISTS mutation_tests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    source_code TEXT NOT NULL,
    language VARCHAR(50) NOT NULL DEFAULT 'rust',
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

-- Create mutation_results table
CREATE TABLE IF NOT EXISTS mutation_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mutation_test_id UUID NOT NULL REFERENCES mutation_tests(id) ON DELETE CASCADE,
    mutation_type VARCHAR(100) NOT NULL,
    original_code TEXT NOT NULL,
    mutated_code TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    column_number INTEGER,
    test_result VARCHAR(50) NOT NULL DEFAULT 'pending',
    execution_time_ms BIGINT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_mutation_tests_status ON mutation_tests(status);
CREATE INDEX IF NOT EXISTS idx_mutation_tests_created_at ON mutation_tests(created_at);
CREATE INDEX IF NOT EXISTS idx_mutation_results_test_id ON mutation_results(mutation_test_id);
CREATE INDEX IF NOT EXISTS idx_mutation_results_result ON mutation_results(test_result);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply updated_at triggers
DROP TRIGGER IF EXISTS update_mutation_tests_updated_at ON mutation_tests;
CREATE TRIGGER update_mutation_tests_updated_at BEFORE UPDATE ON mutation_tests
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_mutation_results_updated_at ON mutation_results;
CREATE TRIGGER update_mutation_results_updated_at BEFORE UPDATE ON mutation_results
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
