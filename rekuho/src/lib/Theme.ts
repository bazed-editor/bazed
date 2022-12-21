export type Font = {
  family: string
  size: string
  weight: string
}

export type Gutter = {
  background: string
  width: number
}

export type Theme = {
  font: Font
  gutter: Gutter

  scrollbar_width: number
  text_color: string
  text_offset: number
  primary_cursor_color: string
  editor_background: string
}

import kanagawa from "./kanagawa"

export const example: Theme = {
  font: {
    family: "monospace",
    weight: "normal",
    size: "20px",
  },

  gutter: {
    background: kanagawa.sumiInk0,
    width: 50,
  },

  scrollbar_width: 12,
  text_offset: 10,
  text_color: kanagawa.fujiWhite,
  primary_cursor_color: kanagawa.fujiWhite,
  editor_background: kanagawa.sumiInk1,
}
