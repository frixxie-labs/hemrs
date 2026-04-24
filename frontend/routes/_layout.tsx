import { Context } from "fresh";

export default function Layout(ctx: Context<unknown>) {
  return (
    <div class="min-h-screen bg-dark-bg">
      <header class="bg-dark-header border-b border-dark-border">
        <div class="max-w-screen-xl mx-auto px-4 sm:px-6 py-4 flex items-center justify-between">
          <div>
            <span class="text-accent-green text-sm font-semibold tracking-wide uppercase">
              HEMRS
            </span>
            <h1 class="text-xl font-bold text-text-primary">
              Sensor dashboard
            </h1>
          </div>
          <nav class="flex gap-2">
            <a
              class="px-4 py-1.5 text-sm rounded-full border border-dark-border text-text-primary hover:bg-dark-card-inner transition-colors"
              href="/"
            >
              Home
            </a>
            <a
              class="px-4 py-1.5 text-sm rounded-full border border-dark-border text-text-primary hover:bg-dark-card-inner transition-colors"
              href="/devices"
            >
              Devices
            </a>
            <a
              class="px-4 py-1.5 text-sm rounded-full border border-dark-border text-text-primary hover:bg-dark-card-inner transition-colors"
              href="/sensors"
            >
              Sensors
            </a>
            <a
              class="px-4 py-1.5 text-sm rounded-full border border-dark-border text-text-primary hover:bg-dark-card-inner transition-colors"
              href="/measurements"
            >
              Measurements
            </a>
          </nav>
        </div>
      </header>
      <main class="max-w-screen-xl mx-auto px-4 sm:px-6 py-6">
        <ctx.Component />
      </main>
    </div>
  );
}
