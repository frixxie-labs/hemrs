interface MeasurementsInfoProps {
  device_count: number;
  sensor_count: number;
  measurement_count: number;
}

export default function MeasurementsInfo({
  device_count,
  sensor_count,
  measurement_count,
}: MeasurementsInfoProps) {
  return (
    <div class="bg-slate-100 p-3 sm:p-4 rounded shadow-md mt-6 w-full">
      <h2 class="text-lg sm:text-xl font-semibold mb-4 text-gray-800">
        System Overview
      </h2>
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 sm:gap-4">
        <div class="bg-white p-3 rounded-lg text-center">
          <div class="text-2xl sm:text-3xl font-bold text-blue-600">
            {device_count}
          </div>
          <div class="text-sm sm:text-base text-gray-600">Devices</div>
        </div>
        <div class="bg-white p-3 rounded-lg text-center">
          <div class="text-2xl sm:text-3xl font-bold text-green-600">
            {sensor_count}
          </div>
          <div class="text-sm sm:text-base text-gray-600">Sensors</div>
        </div>
        <div class="bg-white p-3 rounded-lg text-center">
          <div class="text-2xl sm:text-3xl font-bold text-purple-600">
            {measurement_count}
          </div>
          <div class="text-sm sm:text-base text-gray-600">Measurements</div>
        </div>
      </div>
    </div>
  );
}
