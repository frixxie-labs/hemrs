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
    <div class="bg-slate-100 p-3 sm:p-4 rounded shadow-md mt-6 w-full">
      <h2 class="text-lg sm:text-xl font-semibold mb-4 text-gray-800">
        Measurement Statistics
      </h2>

      {/* Key stats grid */}
      <div class="grid grid-cols-2 sm:grid-cols-3 gap-3 mb-4">
        <div class="bg-white p-3 rounded-lg text-center">
          <div class="text-lg sm:text-xl font-bold text-red-600">
            {stats.measurement_stats.min}
          </div>
          <div class="text-xs sm:text-sm text-gray-600">Min</div>
        </div>
        <div class="bg-white p-3 rounded-lg text-center">
          <div class="text-lg sm:text-xl font-bold text-green-600">
            {stats.measurement_stats.max}
          </div>
          <div class="text-xs sm:text-sm text-gray-600">Max</div>
        </div>
        <div class="bg-white p-3 rounded-lg text-center col-span-2 sm:col-span-1">
          <div class="text-lg sm:text-xl font-bold text-blue-600">
            {stats.measurement_stats.avg}
          </div>
          <div class="text-xs sm:text-sm text-gray-600">Average</div>
        </div>
      </div>

      {/* Additional stats */}
      <div class="bg-white p-3 rounded-lg space-y-2">
        <div class="flex justify-between items-center">
          <span class="text-sm font-medium text-gray-700">Count:</span>
          <span class="text-sm text-gray-900">
            {stats.measurement_stats.count}
          </span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-sm font-medium text-gray-700">Std Dev:</span>
          <span class="text-sm text-gray-900">
            {stats.measurement_stats.stddev}
          </span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-sm font-medium text-gray-700">Variance:</span>
          <span class="text-sm text-gray-900">
            {stats.measurement_stats.variance}
          </span>
        </div>
        <div class="pt-2 border-t border-gray-200">
          <div class="text-sm font-medium text-gray-700 mb-1">
            Latest Measurement:
          </div>
          <div class="text-sm text-gray-900">
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
