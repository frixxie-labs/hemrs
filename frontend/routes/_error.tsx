import { HttpError, PageProps } from "fresh";

export default function ErrorPage(props: PageProps) {
  const error = props.error;
  if (error instanceof HttpError) {
    if (error.status === 404) {
      return (
        <div class="flex items-center justify-center min-h-[50vh]">
          <h1 class="text-2xl font-bold text-text-primary">
            404 - Page not found
          </h1>
        </div>
      );
    }
  }

  return (
    <div class="flex items-center justify-center min-h-[50vh]">
      <h1 class="text-2xl font-bold text-text-primary">Oh no...</h1>
    </div>
  );
}
