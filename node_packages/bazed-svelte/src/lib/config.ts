import type { Theme } from "./theme"
import { theme as kanagawa } from "./theme/kanagawa"

type sizeCss = string
type fontFamilyCss = string
type fontStyleCss = string
type pixels = number

export type Font = {
  family: fontFamilyCss
  size: sizeCss
  weight: fontStyleCss
}

export type Config = {
  theme: Theme
  font: Font

  gutterWidth: pixels
  scrollbarWidth: pixels
  textOffset: pixels
}

export const example: Config = {
  theme: kanagawa,
  font: {
    family: "monospace",
    weight: "normal",
    size: "20px",
  },

  gutterWidth: 50,
  scrollbarWidth: 12,
  textOffset: 10,
}
