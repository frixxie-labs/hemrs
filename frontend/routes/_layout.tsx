import { Context } from "fresh";

export default function Layout(ctx: Context<unknown>) {
  return (
    <div class="layout bg-slate-50 min-h-screen p-2 sm:p-4 rounded-lg">
      <header class="bg-[#86efac] shadow-md rounded-lg">
        <div class="border-b-2 border-gray-200 rounded-lg flex items-center min-h-16">
          <nav class="flex flex-wrap justify-start items-center px-2">
            <a
              class="mx-1 sm:ml-4 sm:mx-0 hover:text-black hover:underline rounded-full px-2 py-1 text-sm sm:text-base"
              href="/"
            >
              Home
            </a>
            <a
              class="mx-1 sm:ml-4 sm:mx-0 hover:text-black hover:underline rounded-full px-2 py-1 text-sm sm:text-base"
              href="/devices"
            >
              Devices
            </a>
            <a
              class="mx-1 sm:ml-4 sm:mx-0 hover:text-black hover:underline rounded-full px-2 py-1 text-sm sm:text-base"
              href="/sensors"
            >
              Sensors
            </a>
            <a
              class="mx-1 sm:ml-4 sm:mx-0 hover:text-black hover:underline rounded-full px-2 py-1 text-sm sm:text-base"
              href="/measurements"
            >
              Measurements
            </a>
          </nav>
        </div>
      </header>
      <main class="mt-4">
        <ctx.Component />
      </main>
    </div>
  );
}
