import clsx from "clsx";

type IconButtonProps = {
  icon: React.ReactNode;
  color?: "info" | "error" | "primary" | "success" | "warning";
} & React.ComponentProps<"a">;

export default function IconButton({ icon, color, ...rest }: IconButtonProps) {
  color = color ?? "info";
  const className = clsx(
    `color-${color}-text hover:color-${color}-text-hover active:color-${color}-text-active`,
    "block active:transform-scale-95 transition-duration-300 ease-in-out"
  );

  return (
    <a className={className} {...rest}>
      {icon}
    </a>
  );
}
