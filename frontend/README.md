# HEMRS frontend

Device and sensor monitor built with Fresh, Deno and Preact.

## Development

From this directory:

```sh
HEMRS_URL=http://localhost:65534/ PLOTTER_URL=http://localhost:8000/ deno task dev
```

Open the URL printed by Vite. `HEMRS_URL` must include a trailing slash.

The overview shows counts, measurement history, latest-value plots, latest
readings, and searchable devices grouped by location. Readings on the overview
are a snapshot at page load. Device sensor pages include live updates,
statistics and 24-hour plots. Unavailable plots show a fallback message.

The monitor layout combines location-grouped devices with compact measurement
tables and uses the existing Kanagawa theme. It is the default UI; no preview
parameters or sample data are used.

## Checks and production

```sh
deno task check
deno task test
deno task build
deno task start
```

Production also requires `HEMRS_URL` and optionally `PLOTTER_URL` (defaults to
`http://localhost:8000/`).
