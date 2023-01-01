import type { KeyInput, Key, Modifier } from "./rpc"

export const keyboardToKeyInput = (event: KeyboardEvent): KeyInput | null => {
  const modifiers: Modifier[] = []
  if (event.ctrlKey) modifiers.push("ctrl")
  if (event.shiftKey) modifiers.push("shift")
  if (event.altKey) modifiers.push("alt")
  if (event.metaKey) modifiers.push("win")

  let key: Key | null = null
  if (event.key.length === 1) {
    key = { char: event.key }
  }

  switch (event.key) {
    case "Enter":
      key = "return"
      break
    case "Backspace":
      key = "backspace"
      break
    case "ArrowLeft":
      key = "left"
      break
    case "ArrowRight":
      key = "right"
      break
    case "ArrowUp":
      key = "up"
      break
    case "ArrowDown":
      key = "down"
      break
    case "Escape":
      key = "escape"
      break
  }

  return key ? { modifiers, key } : null
}
