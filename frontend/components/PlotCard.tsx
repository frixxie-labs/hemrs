interface PlotCardProps {
  title: string;
  svg: string | null;
}

export default function PlotCard({ title, svg }: PlotCardProps) {
  if (!svg) {
    return (
      <div class="w-full bg-gray-50 border border-gray-200 rounded-lg p-4 text-center text-gray-500">
        <p>{title} — plot unavailable</p>
      </div>
    );
  }
  return (
    <div class="w-full bg-white border border-gray-200 rounded-lg p-4">
      <h2 class="text-lg font-semibold text-gray-700 mb-2">{title}</h2>
      <div class="w-full overflow-x-auto">
        <img src={svg} alt={title} class="w-full" />
      </div>
    </div>
  );
}
