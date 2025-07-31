-- Rollback: Drop mutation_results and mutation_tests tables

DROP TABLE IF EXISTS mutation_results;
DROP TABLE IF EXISTS mutation_tests;
DROP TYPE IF EXISTS mutation_test_status;
DROP TYPE IF EXISTS test_result;