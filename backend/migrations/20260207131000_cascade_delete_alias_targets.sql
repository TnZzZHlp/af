-- Enable recursive delete for alias targets when an alias is deleted
ALTER TABLE alias_targets
DROP CONSTRAINT alias_targets_alias_id_fkey,
ADD CONSTRAINT alias_targets_alias_id_fkey
FOREIGN KEY (alias_id)
REFERENCES aliases(id)
ON DELETE CASCADE;
