import { Handlers, PageProps } from "$fresh/server.ts";

interface Data {
  ts: number;
  room: string;
  temperature: number;
  humidity: number;
}

export const handler: Handlers<Array<Data>> = {
  async GET(_req, ctx) {
    const data: Array<Data> = await fetch("http://localhost:65534/").then((
      r,
    ) => r.json());
    if (!data) {
      return new Response("Data not found", { status: 404 });
    }
    return ctx.render(data);
  },
};

export default function DataPage(props: PageProps<Array<Data>>) {
  return (
    <div>
      <h1>{props.data[props.data.length - 1].room}</h1>
      <p>{props.data[props.data.length - 1].temperature}</p>
      <p>{props.data[props.data.length - 1].humidity}</p>
      <p>{props.data[props.data.length - 1].ts}</p>
    </div>
  );
}
