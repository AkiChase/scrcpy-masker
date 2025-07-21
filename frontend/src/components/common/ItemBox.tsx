import { Flex } from "antd";
import type { ComponentProps, PropsWithChildren, ReactNode } from "react";

type ItemBoxContainerProps = PropsWithChildren<{
  gap?: number;
}> &
  ComponentProps<"div">;

export function ItemBoxContainer({
  children,
  gap,
  ...rest
}: ItemBoxContainerProps) {
  gap = gap ?? 24;

  return (
    <Flex {...rest} vertical gap={gap}>
      {children}
    </Flex>
  );
}

type ItemBoxProps = PropsWithChildren<{
  label?: ReactNode;
}> &
  ComponentProps<"div">;

export function ItemBox({ label, children, ...rest }: ItemBoxProps) {
  return (
    <div {...rest}>
      {label && (
        <div className="color-text font-bold pb-2 pl-1 pr-1">{label}</div>
      )}
      <div>{children}</div>
    </div>
  );
}
