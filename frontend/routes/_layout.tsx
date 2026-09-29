import { Context } from "fresh";

const links = [["/", "Overview"], ["/devices", "Devices"], [
  "/sensors",
  "Sensors",
], ["/measurements", "Measurements"]];

export default function Layout(ctx: Context<unknown>) {
  return (
    <div class="monitor-app">
      <div class="monitor-shell">
        <header class="monitor-header">
          <a class="monitor-brand" href="/">
            HEMRS <span>Device & sensor monitor</span>
          </a>
          <nav aria-label="Main navigation">
            {links.map(([href, label]) => {
              const active = ctx.url.pathname === href ||
                (href !== "/" && ctx.url.pathname.startsWith(`${href}/`));
              return (
                <a
                  key={href}
                  href={href}
                  aria-current={active ? "page" : undefined}
                >
                  {label}
                </a>
              );
            })}
          </nav>
        </header>
        <main class="monitor-main">
          <ctx.Component />
        </main>
      </div>
    </div>
  );
}
