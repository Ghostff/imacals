CREATE TABLE IF NOT EXISTS files (
    id             UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    created_by     UUID         NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    fileable_type  VARCHAR NOT NULL,
    fileable_id    UUID         NOT NULL,
    type           VARCHAR NOT NULL,
    name           VARCHAR NOT NULL,
    absolute_path  TEXT         NOT NULL,
    relative_path  TEXT         NOT NULL,
    size           BIGINT       NOT NULL,
    mime_type      VARCHAR NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at     TIMESTAMPTZ
);

-- =========================
-- Indexes
-- =========================

-- Primary lookup: all files for a given owner row.
CREATE INDEX IF NOT EXISTS files_fileable_index
    ON files (fileable_type, fileable_id)
    WHERE deleted_at IS NULL;

-- Type-scoped lookup: e.g. fetch only the signature for a user.
CREATE INDEX IF NOT EXISTS files_fileable_type_index
    ON files (fileable_type, fileable_id, type)
    WHERE deleted_at IS NULL;

-- FK: uploader.
CREATE INDEX IF NOT EXISTS files_created_by_index
    ON files (created_by)
    WHERE deleted_at IS NULL;

-- Soft-delete filtering for audit / restore queries.
CREATE INDEX IF NOT EXISTS files_deleted_at_index
    ON files (deleted_at);
