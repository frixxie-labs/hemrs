interface PlotCardProps {
  title: string;
  svg: string | null;
}

export default function PlotCard({ title, svg }: PlotCardProps) {
  if (!svg) {
    return (
      <div class="w-full bg-dark-card border border-dark-border rounded-xl p-4 text-center text-text-muted">
        <p>{title} — plot unavailable</p>
      </div>
    );
  }
  return (
    <div class="w-full bg-dark-card border border-dark-border rounded-xl p-4">
      <h2 class="text-lg font-semibold text-text-primary mb-2">{title}</h2>
      <div class="w-full overflow-x-auto">
        <img src={svg} alt={title} class="w-full" />
      </div>
    </div>
  );
}
