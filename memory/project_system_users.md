---
name: System Users Feature
description: domain_system_users table — one named user per (domain, job-title role), for when the system needs a real human for a region
type: project
---

System users are per-domain assignments of a real platform user to a job-title role — the person
the system acts through when it needs a named human for that region.

**Why:** Regional responsibility varies by location. The same mechanism that once named a broker and
realtor per domain is the right shape for naming, say, a delivery manager or accounts contact per
market.

**How to apply:** Resolve the domain for the location first, then look up `domain_system_users` for
that domain. Falls back to the country-level domain when there is no city/state match.

Key facts:
- Table: `domain_system_users` (domain_id, **user_role_id**, user_id, created_by, soft-delete).
  The original free-text `role` column was replaced by an FK to `organization_user_role` in
  `20260511300000_refactor_domain_system_users_role_fk` — that migration drops and recreates the
  table, so anything written against `role` is stale.
- Eligibility is a flag on the role, not a hardcoded list: `organization_user_role.system_user_eligible`.
  Currently true for `hml`, `insurance`, `broker`, `realtor` — the renovation-era set the codebase
  started from. It needs to change alongside the job titles in `docs/business_logic.md §1b`.
- Unique partial index: one active assignment per `(domain_id, user_role_id)` where `deleted_at IS NULL`.
- `POST /api/domain-system-users` — upsert (soft-deletes the previous holder of that slot)
- `DELETE /api/domain-system-users/{id}` — soft-delete
- `GET /api/domain-system-users/eligible-roles` — the roles that may be assigned
- Only superusers may write; any authenticated user may read
- Dashboard: Users > System Users (`/users/system`) — `SystemUsersView.vue`
- Service: `imacals-dashboard/src/services/systemUser.ts`
