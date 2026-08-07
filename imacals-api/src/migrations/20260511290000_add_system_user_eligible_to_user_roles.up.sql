-- Marks which job-title roles may be used as system users on domain_system_users.
-- Only hml, insurance, broker, and realtor are eligible.
ALTER TABLE organization_user_role
    ADD COLUMN system_user_eligible BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE organization_user_role
SET    system_user_eligible = TRUE
WHERE  name IN ('hml', 'insurance', 'broker', 'realtor');
