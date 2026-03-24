# Events

<!-- This is a living document serving as a registry of event types in your project.
     It starts mostly empty and grows as features publish or subscribe to events.
     Add new entries to the Event Catalog as each feature is built.

     Consider specifying retry policy and dead-letter handling as dedicated
     feature specs rather than embedding them here. -->

## Envelope Format

<!-- The standard wrapper structure for all events. Example:

All events use a common envelope:

```json
{
  "id": "evt_01J...",
  "type": "order.created",
  "timestamp": "2025-01-15T09:30:00Z",
  "data": {}
}
```

The `data` field contains the event-specific payload documented in the catalog below.

-->

## Naming Convention

<!-- How event types and subjects are named. Example:

Event types use dot notation: `{resource}.{action}` (e.g., `order.created`,
`user.email_changed`).

Subject or topic names follow the pattern `{service}.{resource}.{action}`.

-->

## Event Catalog

<!-- Registry of event types. Add entries as features are built.
     Each entry documents the event type, its payload, and who publishes
     and subscribes to it. Example:

### order.created

Published when a new order is placed.

- **Publisher**: orders module
- **Subscribers**: notifications, billing
- **Payload**:

```json
{
  "order_id": "ord_01J...",
  "customer_id": "cus_01J...",
  "total": 4999,
  "currency": "usd"
}
```

### user.email_changed

Published when a user updates their email address.

- **Publisher**: users module
- **Subscribers**: notifications, audit
- **Payload**:

```json
{
  "user_id": "usr_01J...",
  "old_email": "old@example.com",
  "new_email": "new@example.com"
}
```

-->
