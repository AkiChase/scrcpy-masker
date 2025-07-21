export function mappingButtonPresetStyle(radius: number): React.CSSProperties {
  return {
    width: radius,
    height: radius,
    left: -radius / 2,
    top: -radius / 2,
  };
}

export function clientPositionToMappingPosition(
  cX: number,
  cY: number,
  mL: number,
  mT: number,
  mW: number,
  mH: number,
  oW: number,
  oH: number
) {
  const mX = Math.max(0, Math.min(mW, cX - mL));
  const mY = Math.max(0, Math.min(mH, cY - mT));
  return {
    x: Math.round((mX / mW) * oW),
    y: Math.round((mY / mH) * oH),
  };
}

export function mappingButtonPosition(
  oX: number,
  oY: number,
  oW: number,
  oH: number,
  mW: number,
  mH: number
) {
  return {
    x: Math.round((oX / oW) * mW),
    y: Math.round((oY / oH) * mH),
  };
}

export function mappingButtonTransformStyle(
  oX: number,
  oY: number,
  oW: number,
  oH: number,
  mW: number,
  mH: number
): string {
  const x = Math.round((oX / oW) * mW);
  const y = Math.round((oY / oH) * mH);
  return `translate(${x}px, ${y}px)`;
}

export function mappingButtonDragFactory(
  maskArea: { width: number; height: number; left: number; top: number },
  originalSize: { width: number; height: number },
  onMouseUp: ({ x, y }: { x: number; y: number }) => void,
  delay?: number
) {
  delay = delay ?? 500;
  const handleDrag = (downEvent: React.MouseEvent) => {
    if (downEvent.button !== 0) return;

    const { width, height, left, top } = maskArea;
    const element = downEvent.currentTarget as HTMLElement;

    let dragStarted = false;
    let longPressTimer = 0;
    let curMaskX = 0;
    let curMaskY = 0;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      curMaskX = Math.max(0, Math.min(moveEvent.clientX - left, width));
      curMaskY = Math.max(0, Math.min(moveEvent.clientY - top, height));
      if (!dragStarted) return;
      element.style.transform = `translate(${curMaskX}px, ${curMaskY}px)`;
    };

    const handleMouseUp = (upEvent: MouseEvent) => {
      clearTimeout(longPressTimer);
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
      if (!dragStarted) return;

      curMaskX = Math.max(0, Math.min(upEvent.clientX - left, width));
      curMaskY = Math.max(0, Math.min(upEvent.clientY - top, height));
      element.style.transform = `translate(${curMaskX}px, ${curMaskY}px)`;

      onMouseUp({
        x: Math.round((curMaskX / width) * originalSize.width),
        y: Math.round((curMaskY / height) * originalSize.height),
      });
    };

    curMaskX = Math.max(0, Math.min(downEvent.clientX - left, width));
    curMaskY = Math.max(0, Math.min(downEvent.clientY - top, height));
    window.addEventListener("mousemove", handleMouseMove);

    longPressTimer = setTimeout(() => {
      dragStarted = true;
      element.style.transform = `translate(${curMaskX}px, ${curMaskY}px)`;
    }, delay);
    window.addEventListener("mouseup", handleMouseUp);
  };

  return handleDrag;
}
