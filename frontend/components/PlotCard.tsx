interface PlotCardProps {
  title: string;
  svg: string | null;
}

export default function PlotCard({ title, svg }: PlotCardProps) {
  return (
    <section class="monitor-plot">
      <div class="monitor-section-heading">
        <h2>{title}</h2>
      </div>
      {svg
        ? (
          <div class="w-full overflow-x-auto">
            <img src={svg} alt={title} class="w-full" />
          </div>
        )
        : <p class="monitor-empty">Plot unavailable.</p>}
    </section>
  );
}
