import { JSX } from "preact";
import type { Signal } from "@preact/signals";

export type ButtonProps =
  & Omit<JSX.HTMLAttributes<HTMLButtonElement>, "type">
  & {
    type?:
      | "submit"
      | "reset"
      | "button"
      | undefined
      | Signal<"submit" | "reset" | "button" | undefined>;
  };

export default function Button(props: ButtonProps) {
  return (
    <button
      {...props}
      class={`px-2 py-1 border-gray-500 border-2 rounded bg-white hover:bg-gray-200 transition-colors ${
        props.class ?? ""
      }`.trim()}
    />
  );
}
