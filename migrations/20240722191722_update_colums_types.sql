-- Update column types

ALTER TABLE users
ALTER COLUMN permissions TYPE NUMERIC,
ALTER COLUMN flags TYPE NUMERIC
