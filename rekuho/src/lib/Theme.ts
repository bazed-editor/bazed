export type Font = {
  family: string
  size: string
  weight: string
}

export type Theme = {
  font: Font

  font_color: string
  gutter_background: string
  primary_cursor_color: string
  editor_background: string
}

import kanagawa from "./kanagawa"

export let example: Theme = {
  font: {
    family: "monospace",
    weight: "normal",
    size: "20px",
  },
  text_offset: 10,
  gutter: {
    background: kanagawa.sumiInk0,
    width: 50,
  },

  font_color: kanagawa.fujiWhite,
  primary_cursor_color: kanagawa.fujiWhite,
  editor_background: kanagawa.sumiInk1,
}
