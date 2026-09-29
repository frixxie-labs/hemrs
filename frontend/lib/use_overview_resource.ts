import { useEffect, useState } from "preact/hooks";

/** Load each section independently and cancel requests when it is replaced. */
export function useOverviewResource<T>(url: string | null) {
  const [result, setResult] = useState<
    {
      url: string;
      data?: T;
      error?: boolean;
    } | null
  >(null);
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    if (!url) return;
    const controller = new AbortController();
    setResult(null);
    fetch(url, {
      signal: AbortSignal.any([controller.signal, AbortSignal.timeout(60_000)]),
    }).then(async (response) => {
      if (!response.ok) throw new Error("Data unavailable");
      const data: T = await response.json();
      if (!controller.signal.aborted) setResult({ url, data });
    }).catch(() => {
      if (!controller.signal.aborted) setResult({ url, error: true });
    });
    return () => controller.abort();
  }, [url, attempt]);

  const current = result?.url === url ? result : null;
  return {
    data: current?.data,
    error: current?.error ?? false,
    retry: () => setAttempt((value) => value + 1),
  };
}
