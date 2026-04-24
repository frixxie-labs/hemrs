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
      class={`px-4 py-1.5 border border-dark-border rounded-full bg-dark-card-inner text-text-primary hover:bg-dark-border transition-colors ${
        props.class ?? ""
      }`.trim()}
    />
  );
}
