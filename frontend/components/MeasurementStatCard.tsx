import { MeasurementStats } from "../lib/measurement_stats.ts";
import { Measurement } from "../lib/measurements.ts";

interface MeasurementStatsCardProps {
  measurement_stats: MeasurementStats;
  latest: Measurement;
}

export default function MeasurementsStatsCard(
  stats: MeasurementStatsCardProps,
) {
  const latestDate = new Date(stats.latest.timestamp);
  const dateStr = latestDate.toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
  const timeStr = latestDate.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
    second: "2-digit",
    hour12: true,
  });
  const count = Number(stats.measurement_stats.count).toLocaleString();

  return (
    <div class="bg-dark-card border border-dark-border rounded-xl p-6 w-full">
      {/* Header */}
      <h2 class="text-xl font-bold text-text-primary mb-5 flex items-center gap-3">
        <span class="text-purple-400 text-2xl">&#x1F4C8;</span>
        Measurement Statistics
      </h2>

      {/* Min / Max / Avg cards */}
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-5">
        {/* Minimum */}
        <div class="bg-dark-card-inner border border-dark-border rounded-lg p-4">
          <div class="flex items-center gap-2 mb-1">
            <span class="inline-flex items-center justify-center w-8 h-8 rounded-full bg-red-400/20 text-red-400 text-sm font-bold">
              &darr;
            </span>
            <span class="text-sm text-text-muted">Minimum</span>
          </div>
          <div class="text-3xl font-bold text-red-400 mb-1">
            {stats.measurement_stats.min}
          </div>
          <div class="text-xs text-text-muted">Lowest measurement recorded</div>
        </div>

        {/* Maximum */}
        <div class="bg-dark-card-inner border border-dark-border rounded-lg p-4">
          <div class="flex items-center gap-2 mb-1">
            <span class="inline-flex items-center justify-center w-8 h-8 rounded-full bg-green-400/20 text-green-400 text-sm font-bold">
              &uarr;
            </span>
            <span class="text-sm text-text-muted">Maximum</span>
          </div>
          <div class="text-3xl font-bold text-green-400 mb-1">
            {stats.measurement_stats.max}
          </div>
          <div class="text-xs text-text-muted">
            Highest measurement recorded
          </div>
        </div>

        {/* Average */}
        <div class="bg-dark-card-inner border border-dark-border rounded-lg p-4">
          <div class="flex items-center gap-2 mb-1">
            <span class="inline-flex items-center justify-center w-8 h-8 rounded-full bg-purple-400/20 text-purple-400 text-sm font-bold">
              =
            </span>
            <span class="text-sm text-text-muted">Average</span>
          </div>
          <div class="text-3xl font-bold text-purple-400 mb-1">
            {stats.measurement_stats.avg}
          </div>
          <div class="text-xs text-text-muted">
            Average of all measurements
          </div>
        </div>
      </div>

      {/* Summary + Latest Measurement */}
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-5">
        {/* Summary */}
        <div class="bg-dark-card-inner border border-dark-border rounded-lg p-4">
          <h3 class="text-base font-bold text-text-primary mb-4 flex items-center gap-2">
            <span class="text-purple-400">&#x2630;</span> Summary
          </h3>
          <div class="space-y-3">
            <div class="flex justify-between items-center">
              <span class="flex items-center gap-2 text-sm text-text-secondary">
                <span class="inline-flex items-center justify-center w-6 h-6 rounded bg-yellow-500/20 text-yellow-400 text-xs font-bold">
                  #
                </span>
                Count
              </span>
              <span class="text-sm text-text-primary font-medium">{count}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="flex items-center gap-2 text-sm text-text-secondary">
                <span class="inline-flex items-center justify-center w-6 h-6 rounded bg-yellow-500/20 text-yellow-400 text-xs font-bold">
                  &sigma;
                </span>
                Standard Deviation
              </span>
              <span class="text-sm text-text-primary font-medium">
                {stats.measurement_stats.stddev}
              </span>
            </div>
            <div class="flex justify-between items-center">
              <span class="flex items-center gap-2 text-sm text-text-secondary">
                <span class="inline-flex items-center justify-center w-6 h-6 rounded bg-yellow-500/20 text-yellow-400 text-xs font-bold">
                  x&sup2;
                </span>
                Variance
              </span>
              <span class="text-sm text-text-primary font-medium">
                {stats.measurement_stats.variance}
              </span>
            </div>
          </div>
        </div>

        {/* Latest Measurement */}
        <div class="bg-dark-card-inner border border-dark-border rounded-lg p-4">
          <h3 class="text-base font-bold text-text-primary mb-4 flex items-center gap-2">
            <span class="text-purple-400">&#x23F0;</span> Latest Measurement
          </h3>
          <div class="bg-dark-card border border-dark-border rounded-lg p-4 text-center mb-3">
            <div class="text-3xl font-bold text-green-400">
              {stats.latest.value}
            </div>
            <div class="text-xs text-text-muted mt-1">
              Latest recorded value
            </div>
          </div>
          <div class="grid grid-cols-2 gap-3">
            <div class="bg-dark-card border border-dark-border rounded-lg p-3 flex items-center gap-2">
              <span class="text-text-muted text-sm">&#x1F4C5;</span>
              <span class="text-sm text-text-primary">{dateStr}</span>
            </div>
            <div class="bg-dark-card border border-dark-border rounded-lg p-3 flex items-center gap-2">
              <span class="text-text-muted text-sm">&#x1F552;</span>
              <span class="text-sm text-text-primary">{timeStr}</span>
            </div>
          </div>
        </div>
      </div>

      {/* About footer */}
      <div class="bg-dark-card-inner border border-blue-500/30 rounded-lg p-4 flex items-start gap-3">
        <span class="inline-flex items-center justify-center w-8 h-8 rounded-full bg-blue-500/20 text-blue-400 text-sm font-bold flex-shrink-0">
          i
        </span>
        <div>
          <div class="text-sm font-semibold text-purple-400">
            About These Statistics
          </div>
          <div class="text-xs text-text-muted">
            Statistics are calculated from {count}{" "}
            measurements. Values are rounded for readability.
          </div>
        </div>
      </div>
    </div>
  );
}
