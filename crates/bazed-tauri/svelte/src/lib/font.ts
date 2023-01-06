import type { Font } from "./theme"

export const fontToString = (font: Font): string => {
  return `${font.weight} ${font.size} ${font.family}`
}

export type Metrics = {
  width: number | undefined
  actualHeight: number | undefined
  height: number | undefined
}

/**
 * using a canvas approach, measure width and height of monospace font characters
 * @param font - in css format
 * @returns calculated bounding box sizes
 */
export const measure = (font: string): Metrics => {
  const canvas = new OffscreenCanvas(0, 0)
  const context = canvas.getContext("2d") as OffscreenCanvasRenderingContext2D | null
  if (context) {
    context.font = font
  }

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

import { elt, removeChildrenAndAdd } from "./common"

/**
 * using a dom-based aproach, measure width and height of monospace font characters
 * @param element - to which to add child to measure
 * @param font - in css format
 * @returns size of box bounding some amount of text using `font`
 */
export const measureOnChild = (element: HTMLElement, font: string): Metrics => {
  const ruler = element.appendChild(elt("div", null, "rekuho-measure"))
  ruler.style.visibility = "hidden"
  ruler.style.whiteSpace = "nowrap"
  ruler.style.position = "absolute"

  const width = computeAsciiWidth(ruler, font)
  const height = computeLineHeight(ruler, font)
  return { width, height, actualHeight: height }
}

/**
 * using a `span` element measure monospace character width
 * @param element - to which to add a span with multiple characters
 * @param font - in css format
 * @returns width of a single character in span of multiple characters of `font`
 */
const computeAsciiWidth = (element: HTMLElement, font: string): number => {
  const anchor = elt("span", "x".repeat(10), null, `font: ${font}`)
  const pre = elt("pre", [anchor])

  removeChildrenAndAdd(element, pre)

  const rect = anchor.getBoundingClientRect()
  const width = (rect.right - rect.left) / 10
  return width || 10
}

/**
 * using a `pre` element measure height of a line
 * @param element - to which to add a span with a character
 * @param font - in css format
 * @returns height of a line using a specific line
 */
const computeLineHeight = (element: HTMLElement, font: string): number => {
  const pre = elt("pre", null, null, `font: ${font}`)
  for (let i = 0; i < 49; ++i) {
    pre.appendChild(document.createTextNode("x"))
    pre.appendChild(elt("br"))
  }
  pre.appendChild(document.createTextNode("x"))

  removeChildrenAndAdd(element, pre)

  const height = pre.offsetHeight / 50
  return height || 1
}
