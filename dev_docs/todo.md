# Repository Improvement TODOs

## High Priority

- Fix frontend helper return types. Several helpers return `null` in catch paths while typed as non-null, for example `Promise<Measurement>`. Update types to include `null` or remove null returns and handle errors consistently.
- Normalize frontend base URLs. `HEMRS_URL` and `PLOTTER_URL` are string-concatenated and require trailing slashes; add a URL join helper or normalize env values once.
- Add backend 404 handling. Missing database rows currently flow through `anyhow` into `500`; map `sqlx::Error::RowNotFound` to `404` where appropriate.
- Revisit `device_sensors` freshness. Direct measurement inserts can leave the materialized view stale until refresh; decide whether to refresh after insert, use `REFRESH MATERIALIZED VIEW CONCURRENTLY`, replace the view, or document eventual consistency as intentional.
- Fix or remove `just integration_test`. It references `backend/backend.hurl`, which is absent in this tree.

## Medium Priority

- Consolidate SQLx metadata. There are `.sqlx/` entries at the repo root and under `backend/.sqlx/`; decide which location is authoritative and remove duplication if safe.
- Add backend router-level integration tests. Current tests mostly call handlers/domain methods directly; add coverage for routing, status codes, and JSON shapes.
- Standardize frontend API error handling. Helpers currently mix `throw`, `console.error`, `[]`, `0`, and `null`; choose consistent patterns and make pages handle errors explicitly.
- Implement or remove the `DeviceList` search UI. The `searchable` prop renders an input but does not filter results.
- Add an OpenAPI validation/check workflow. Route changes should fail fast if `utoipa` registration or schema coverage is missed.

## Lower Priority

- Update stale/empty service READMEs. `frontend/README.md` is default Fresh text and `plotter/README.md` is empty; link them to `dev_docs` or add minimal service-specific commands.
- Clean up spelling/naming issues such as `MeasuremetList` and `lables`.
- Expand plotter cache behavior tests as new chart endpoints are added.
- Add Docker Compose health checks so `plotter` waits for a ready backend rather than only container startup.
- Add a local environment example for `HEMRS_URL`, `PLOTTER_URL`, `BACKEND_URL`, and `DATABASE_URL`.
