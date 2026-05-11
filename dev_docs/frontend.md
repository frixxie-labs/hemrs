# Frontend

## Technology

- Fresh 2 on Deno.
- Preact JSX with `jsx: "precompile"` and `jsxImportSource: "preact"`.
- Tailwind CSS v4 imported from `frontend/assets/styles.css`.
- Server-side route handlers fetch backend/plotter data before rendering pages.

## App Setup

- `frontend/main.ts` creates `new App<State>()`, serves static files, installs a simple request logger middleware, and enables file-system routes with `app.fsRoutes()`.
- `frontend/utils.ts` defines Fresh `State` and exports `define = createDefine<State>()`.
- `frontend/routes/_app.tsx` defines the HTML document shell, page title, viewport metadata, stylesheet link, and Fresh partial body.
- `frontend/routes/_layout.tsx` defines the dashboard layout with header navigation and main content container.

## Environment

- `HEMRS_URL`: backend base URL used by `frontend/lib`. It should include a trailing slash, for example `http://localhost:65534/`.
- `PLOTTER_URL`: plotter base URL used by `frontend/lib/plotter.ts`. Defaults to `http://localhost:8000/`.

## Routes

| Route | File | Purpose |
| --- | --- | --- |
| `/` | `routes/index.tsx` | Dashboard overview and clickable device list. |
| `/devices` | `routes/devices/index.tsx` | Device list and link to create form. |
| `/devices/new` | `routes/devices/new.tsx` | Create device form. |
| `/devices/:device_id` | `routes/devices/[device_id]/index.tsx` | Device detail and sensors observed for that device. |
| `/devices/:device_id/sensors/:sensor_id` | `routes/devices/[device_id]/sensors/[sensor_id].tsx` | Sensor detail for a device, stats, latest value, and plot. |
| `/sensors` | `routes/sensors/index.tsx` | Sensor list and link to create form. |
| `/sensors/new` | `routes/sensors/new.tsx` | Create sensor form. |
| `/measurements` | `routes/measurements/index.tsx` | Latest measurements per device/sensor pair. |
| `/measurements/new` | `routes/measurements/new.tsx` | Create measurement form. |
| `/ping` | `routes/ping.ts` | Frontend local ping endpoint returning `pong`. |

## Data Access Helpers

Use `frontend/lib` for backend and plotter calls rather than calling `fetch` directly from components.

### `lib/device.ts`

- `Device` interface: `id`, `name`, `location`.
- `getDevices()` returns `Device[]`, falling back to `[]` on failure.
- `getDeviceById(device_id)` returns `Device | null`.
- `createDevice(name, location)` posts to `/api/devices` and expects text `OK`.
- `deleteDevice(device)` deletes through `/api/devices` and expects text `OK`.

### `lib/sensor.ts`

- `Sensor` interface: `id`, `name`, `unit`.
- `getSensors()` returns `Sensor[]`, falling back to `[]` on failure.
- `getSensorById(sensorId)` returns `Sensor | null`.
- `getSensorsByDeviceId(deviceId)` calls `/api/devices/{deviceId}/sensors`.
- `createSensor(name, unit)` posts to `/api/sensors` and expects text `OK`.
- `deleteSensor(sensor)` deletes through `/api/sensors` and expects text `OK`.

### `lib/measurements.ts`

- `Measurement` interface mirrors backend `Measurement` JSON.
- `getLatestMeasurement()` calls `/api/measurements/latest`.
- `getLatestMeasurementByDeviceAndSensorId(device_id, sensor_id)` calls the nested latest route.
- `getAllLatestMeasurements()` calls `/api/measurements/latest/all`.
- `getMeasurementCount()` calls `/api/measurements/count`.
- `createMeasurement(device, sensor, measurement)` posts to `/api/measurements`.

### `lib/measurement_stats.ts`

- `MeasurementStats` mirrors backend stats JSON.
- `getMeasurementStats(device_id, sensor_id)` calls the nested stats route.

### `lib/plotter.ts`

- `fetchPlotSvg(path)` fetches SVG bytes from the plotter, base64-encodes them, and returns a `data:image/svg+xml;base64,...` URL.
- Specific helpers generate plotter paths for latest all, all measurements, device measurements, device/sensor measurements, and last-24-hours device/sensor measurements.

## Components

### `Button.tsx`

Shared styled button. It accepts normal button attributes except `type`, which is narrowed to valid button type values and supports Preact signals.

### `DeviceList.tsx`

Displays devices in a mobile card layout and desktop table layout. Optional props:

- `clickable`: wraps entries in links to `/devices/{id}`.
- `searchable`: renders a search input; the current component only renders the input and does not implement filtering.

### `SensorList.tsx`

Displays sensors in mobile cards and desktop table layout. If `device_id` is provided, rows link to `/devices/{device_id}/sensors/{sensor.id}`.

### `MeasurementsList.tsx`

Displays a table of measurements with localized timestamp formatting.

### `MeasurementsInfo.tsx`

Displays dashboard summary cards for device count, sensor count, measurement count, and optional online-device summary.

### `MeasurementStatCard.tsx`

Displays min, max, average, count, standard deviation, variance, and latest measurement metadata.

### `PlotCard.tsx`

Displays a plot image from a data URL or a fallback message when the plot is unavailable.

## Styling and Design

The design uses a Kanagawa-inspired dark theme defined in `frontend/assets/styles.css` with Tailwind v4 `@theme` tokens:

- Backgrounds: `dark-bg`, `dark-card`, `dark-card-inner`, `dark-header`.
- Borders: `dark-border`.
- Text: `text-primary`, `text-secondary`, `text-muted`.
- Accents: `accent-green`, `accent-blue`, `accent-violet`, `accent-cyan`, `accent-yellow`, `accent-orange`, `accent-red`, `accent-pink`, `accent-aqua`.

Prefer existing tokens over hard-coded colors. Preserve mobile-first behavior by keeping card/table split patterns where lists need responsive layouts.

## Testing

Frontend tests live in `frontend/tests` and use Deno test plus mock fetch helpers.

Commands:

- `deno task test`: run tests with `--allow-env tests/`.
- `deno task check`: `deno fmt --check . && deno lint . && deno check`.
- `deno task build`: Vite production build.

When adding a new client helper, add or update tests with `installMockFetch` where possible.
