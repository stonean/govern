# {NNN} — {Feature Name} Data Model

<!-- Optional. Generated during the plan phase when the feature introduces or
     modifies domain entities or data structures. Define the structures relevant
     to the feature — database tables, language-level types, or both.

     Include whichever sections apply to the feature.

## Database Tables

```sql
CREATE TABLE example (
    id          BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id   BIGINT      NOT NULL REFERENCES tenants(id),
    name        TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_example_tenant ON example (tenant_id);
```

## Types / Structs

```go
type Example struct {
    ID        int64     `json:"id"`
    TenantID  int64     `json:"tenant_id"`
    Name      string    `json:"name"`
    CreatedAt time.Time `json:"created_at"`
}
```

```typescript
interface Example {
  id: number;
  tenantId: number;
  name: string;
  createdAt: Date;
}
```

## Notes

- `tenant_id` / `TenantID` — all queries must be scoped by tenant
- `name` — unique within a tenant

-->
