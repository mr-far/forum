-- Fixes

ALTER TABLE threads
ADD COLUMN flags TYPE NUMERIC,
DELETE COLUMN likes

