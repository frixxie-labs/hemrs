interface MeasurementsInfoProps {
  device_count: number;
  sensor_count: number;
  measurement_count: number;
  online_count?: number;
}

export default function MeasurementsInfo({
  device_count,
  sensor_count,
  measurement_count,
  online_count,
}: MeasurementsInfoProps) {
  const onlineText = online_count !== undefined
    ? `${online_count}/${device_count} devices online`
    : null;

  return (
    <div class="bg-dark-card border border-dark-border rounded-xl p-6">
      <div class="flex items-start justify-between mb-6">
        <div>
          <span class="text-accent-green text-sm font-semibold">
            System overview
          </span>
          <h2 class="text-2xl sm:text-3xl font-bold text-text-primary mt-1">
            Overview of devices and measurements
          </h2>
          <p class="text-text-secondary mt-2 text-sm">
            A device is considered online when its latest measurement was
            received within the last 15 minutes.
          </p>
        </div>
        {onlineText && (
          <span class="ml-4 shrink-0 px-4 py-1.5 text-sm rounded-full border border-accent-green text-accent-green">
            {onlineText}
          </span>
        )}
      </div>
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <StatCard label="Devices" value={device_count} subtitle="Registered devices" />
        <StatCard label="Sensors" value={sensor_count} subtitle="Configured sensors" />
        <StatCard
          label="Measurements"
          value={measurement_count.toLocaleString()}
          subtitle="Stored readings"
        />
      </div>
    </div>
  );
}

function StatCard(
  { label, value, subtitle }: {
    label: string;
    value: string | number;
    subtitle: string;
  },
) {
  return (
    <div class="bg-dark-card-inner border border-dark-border rounded-lg p-4">
      <div class="text-accent-green text-sm font-medium">{label}</div>
      <div class="text-3xl sm:text-4xl font-bold text-text-primary mt-1">
        {value}
      </div>
      <div class="text-text-muted text-sm mt-1">{subtitle}</div>
    </div>
  );
}
