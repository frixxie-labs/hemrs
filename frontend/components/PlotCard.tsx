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
          <div class="monitor-plot-image">
            <img src={svg} alt={title} />
          </div>
        )
        : <p class="monitor-empty">Plot unavailable.</p>}
    </section>
  );
}
