# Conventions

## General

- Prefer small, direct changes that fit existing structure.
- Keep documentation and tests close to contract changes.
- Preserve existing module boundaries unless a change clearly requires refactoring.
- Use descriptive errors with context at boundaries where failures cross layers.
- Avoid adding compatibility shims unless there is a concrete deployed-data or external-client need.

## Rust Backend

- Keep HTTP concerns in `backend/src/handlers`.
- Keep model structs and SQLx data-access methods in the current domain modules unless introducing a larger data-layer refactor intentionally.
- Use `anyhow::Result` for domain/data methods.
- Use `HandlerError` for HTTP handler errors.
- Add `#[instrument]` to new handlers and meaningful async boundary functions.
- Prefer `sqlx::query!` and `sqlx::query_as!` for compile-time query checking.
- Update `utoipa` annotations and `ApiDoc` registration when adding public endpoints or schemas.
- Use `#[sqlx::test]` for database behavior and `quickcheck` for pure constructor/serialization invariants when useful.
- Keep measurement ingestion asynchronous through the MPSC channel unless intentionally changing ingestion architecture.

## Backend API

- Use JSON for structured request/response bodies.
- Existing create/update/delete device and sensor endpoints return plain text `OK`; preserve this unless updating all clients.
- Existing error responses are plain text through `HandlerError`; document and coordinate any move to JSON errors.
- Measurement query responses are denormalized for display consumers. If IDs are needed, add them deliberately and update frontend/plotter models.
- Date query parameters should use RFC 3339 / ISO 8601 strings compatible with `chrono::DateTime<Utc>`.

## Database

- Add schema changes as new files in `backend/migrations`; do not edit applied migration history unless the project explicitly allows it.
- Keep SQLx offline metadata current when queries change.
- Be careful with `device_sensors`: it is materialized and can become stale until refreshed.
- Consider query ordering part of the contract for charting and UI display.

## Fresh Frontend

- Fetch backend data from route handlers or `frontend/lib`, not inside presentational components.
- Keep reusable visual pieces in `frontend/components`.
- Use `define.handlers` and `define.page<typeof handler>` for typed route data.
- Prefer `Promise.all` for independent server-side fetches.
- Use existing `Button`, list, card, and table patterns before adding new UI primitives.
- Use `class`, not `className`, following existing Preact/Fresh style.
- Use Tailwind theme tokens from `frontend/assets/styles.css` instead of hard-coded colors.
- Preserve mobile behavior for list-heavy components.

## TypeScript Client Helpers

- Keep interfaces in the helper file that owns the corresponding API calls.
- Add one helper per backend/plotter operation rather than scattering URL construction across route files.
- Treat `HEMRS_URL` as requiring a trailing slash unless the helper is changed to normalize it.
- When a helper can return `null`, reflect that in the Promise type.

## Plotter

- Route functions should be small enough to show fetch, group/filter, plot, cache, and return behavior clearly.
- Reuse `_cache_key`, `_fig_to_svg`, `_apply_kanagawa`, and `_kanagawa_color`.
- Close Matplotlib figures through `_fig_to_svg` to avoid leaking resources.
- Keep Pydantic models synchronized with backend JSON.
- Translate backend connectivity and HTTP failures with `_handle_backend_error`.

## Styling

- The project visual language is Kanagawa-inspired dark UI.
- Use rounded cards, subtle borders, muted text, and green/purple/blue accent tones consistently.
- Tables should use `border-dark-border`, `text-text-*`, and `hover:bg-table-row-hover` patterns.
- Forms should use `bg-dark-card`, `bg-dark-card-inner`, `border-dark-border`, and `focus:border-accent-green`.

## Testing Expectations

- Backend route/data changes should have Rust tests when behavior changes.
- Frontend client helper changes should update Deno tests with mock fetches.
- Plotter endpoint/client changes should update pytest coverage.
- Run the smallest relevant verification first, then broader checks if the change spans services.

## Documentation Expectations

- Update `dev_docs/api.md` for route/payload/response/status changes.
- Update `dev_docs/database.md` for migrations, schema changes, and view behavior.
- Update service-specific docs when changing architecture or conventions.
- Update `dev_docs/agent-guide.md` when common workflows or sharp edges change.
