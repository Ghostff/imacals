#!/usr/bin/env bash
set -e  # stop on first error

name="$1"
src="./src/migrations"

if [ -z "$name" ]; then
  echo -e "\033[31m❌  Error:\033[0m migration name is required!"
  echo "Usage: bash migration.sh <migration_name> <?table_name>"
  exit 1
fi

# Run sqlx migrate add and capture the output
output=$(sqlx migrate add --source "$src" -r "$name")
chown -R $USER_UID:$USER_UID "$src"

# Extract migration filename prefix (e.g. 20251016165200_create_users_table)
prefix=$(echo "$output" | sed -n 's|.*migrations/\([0-9]\{14\}_[^\.]*\)\.up\.sql.*|\1|p')
up_file="$src/${prefix}.up.sql"
down_file="$src/${prefix}.down.sql"

# --------------------------
#   Boilerplate Functions
# --------------------------

make_create_boilerplate() {
  local table="$1"
  cat <<EOF
-- Up migration: create_${table}_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "${table}"
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    -- Add your columns here
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ NULL
);
EOF
}

make_create_down_boilerplate() {
  local table="$1"
  cat <<EOF
-- Down migration: create_${table}_table
DROP TABLE IF EXISTS "${table}";
EOF
}

make_alter_boilerplate() {
  local table="$1"
  cat <<EOF
-- Up migration: alter_${table}_table
ALTER TABLE "${table}"
-- Example:
-- ADD COLUMN new_column TEXT DEFAULT '';
;
EOF
}

make_alter_down_boilerplate() {
  local table="$1"
  cat <<EOF
-- Down migration: alter_${table}_table
ALTER TABLE "${table}"
-- Example:
-- DROP COLUMN new_column;
;
EOF
}

make_default_boilerplate() {
  local name="$1"
  cat <<EOF
-- Up migration: $name
-- Write your SQL here
EOF
}

make_default_down_boilerplate() {
  local name="$1"
  cat <<EOF
-- Down migration: $name
-- Write your rollback SQL here
EOF
}


generate_boilerplates() {
  local name="$1"
  # Extract probable table name from pattern like create_users_table
  local table=${2:-$(echo "$name" | sed -E 's/(create_|alter_|add_|update_|_table)//g' | awk '{print $1}')}

  if [[ "$name" == create_* || "$name" == *_create* ]]; then
    up_content=$(make_create_boilerplate "$table")
    down_content=$(make_create_down_boilerplate "$table")
  elif [[ "$name" == alter_* || "$name" == *_alter* || "$name" == add_* || "$name" == *_add* || "$name" == update_* || "$name" == *_update* ]]; then
    up_content=$(make_alter_boilerplate "$table")
    down_content=$(make_alter_down_boilerplate "$table")
  else
    up_content=$(make_default_boilerplate "$name")
    down_content=$(make_default_down_boilerplate "$name")
  fi

  echo "$up_content" > "$up_file"
  echo "$down_content" > "$down_file"

  echo -e "\033[32m✅ Created migration files with boilerplate:\n   • $up_file\n   • $down_file\033[0m"
}

# --------------------------
#   Generate Files
# --------------------------
generate_boilerplates "$name" "$2"
