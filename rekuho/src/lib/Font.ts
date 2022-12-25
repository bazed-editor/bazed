export const measure = (
  font: string,
): {
  width: number | undefined
  actualHeight: number | undefined
  height: number | undefined
} => {
  const canvas = new OffscreenCanvas(0, 0)
  const context = canvas.getContext("2d") as OffscreenCanvasRenderingContext2D | null
  if (context) {
    context.font = font
  }

  // FIXME: elkowar mentioned: "with a font of **ZERO WIDTH X-CHARACTERS**" this breaks.
  // It does. ture. (typo intended)
  const metrics = context?.measureText("ABCDEFGHIJKLMNOPQRSTUVXYZabcdefghijklmnopqrstuvxyz")

  return {
    width: context?.measureText("X").width,
    height:
      metrics?.fontBoundingBoxAscent !== undefined && metrics?.fontBoundingBoxDescent !== undefined
        ? metrics?.fontBoundingBoxAscent + metrics?.fontBoundingBoxDescent
        : undefined,
    actualHeight:
      metrics?.actualBoundingBoxAscent !== undefined &&
      metrics?.actualBoundingBoxDescent !== undefined
        ? metrics?.actualBoundingBoxAscent + metrics?.actualBoundingBoxDescent
        : undefined,
  }
}
