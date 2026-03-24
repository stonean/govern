# {NNN} — {Feature Name} Data Model

<!-- Optional. Generated during the plan phase when the feature involves persistence.
     Define tables, indexes, and column notes. Example:

## {Table Name}

```sql
CREATE TABLE example (
    id          BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id   BIGINT      NOT NULL REFERENCES tenants(id),
    name        TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_example_tenant ON example (tenant_id);
```

## Column Notes

- `tenant_id` — all queries must be scoped by tenant
- `name` — unique within a tenant (enforced by unique index)

-->
