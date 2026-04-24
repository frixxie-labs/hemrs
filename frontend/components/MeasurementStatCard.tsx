import { MeasurementStats } from "../lib/measurement_stats.ts";
import { Measurement } from "../lib/measurements.ts";

interface MeasurementStatsCardProps {
  measurement_stats: MeasurementStats;
  latest: Measurement;
}

export default function MeasurementsStatsCard(
  stats: MeasurementStatsCardProps,
) {
  return (
    <div class="bg-dark-card border border-dark-border rounded-xl p-6 w-full">
      <h2 class="text-xl font-bold text-text-primary mb-4">
        Measurement Statistics
      </h2>

      <div class="grid grid-cols-2 sm:grid-cols-3 gap-3 mb-4">
        <div class="bg-dark-card-inner border border-dark-border rounded-lg p-3 text-center">
          <div class="text-lg sm:text-xl font-bold text-red-400">
            {stats.measurement_stats.min}
          </div>
          <div class="text-xs sm:text-sm text-text-muted">Min</div>
        </div>
        <div class="bg-dark-card-inner border border-dark-border rounded-lg p-3 text-center">
          <div class="text-lg sm:text-xl font-bold text-green-400">
            {stats.measurement_stats.max}
          </div>
          <div class="text-xs sm:text-sm text-text-muted">Max</div>
        </div>
        <div class="bg-dark-card-inner border border-dark-border rounded-lg p-3 text-center col-span-2 sm:col-span-1">
          <div class="text-lg sm:text-xl font-bold text-blue-400">
            {stats.measurement_stats.avg}
          </div>
          <div class="text-xs sm:text-sm text-text-muted">Average</div>
        </div>
      </div>

      <div class="bg-dark-card-inner border border-dark-border rounded-lg p-3 space-y-2">
        <div class="flex justify-between items-center">
          <span class="text-sm font-medium text-text-secondary">Count:</span>
          <span class="text-sm text-text-primary">
            {stats.measurement_stats.count}
          </span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-sm font-medium text-text-secondary">Std Dev:</span>
          <span class="text-sm text-text-primary">
            {stats.measurement_stats.stddev}
          </span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-sm font-medium text-text-secondary">Variance:</span>
          <span class="text-sm text-text-primary">
            {stats.measurement_stats.variance}
          </span>
        </div>
        <div class="pt-2 border-t border-dark-border">
          <div class="text-sm font-medium text-text-secondary mb-1">
            Latest Measurement:
          </div>
          <div class="text-sm text-text-primary">
            <span class="font-semibold">{stats.latest.value}</span> at{" "}
            <span class="text-xs sm:text-sm">
              {new Date(stats.latest.timestamp).toLocaleString()}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
