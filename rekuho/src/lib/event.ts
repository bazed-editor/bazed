import type { KeyInput, Key, Modifier } from "./rpc"

export const getModifiers = (event: WheelEvent | KeyboardEvent): Modifier[] => {
  const modifiers: Modifier[] = []
  if (event.ctrlKey) modifiers.push("ctrl")
  if (event.shiftKey) modifiers.push("shift")
  if (event.altKey) modifiers.push("alt")
  if (event.metaKey) modifiers.push("win")
  return modifiers
}

export const keyboardToKeyInput = (event: KeyboardEvent): KeyInput | null => {
  const modifiers = getModifiers(event)

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

const wheelDelta = (event: WheelEvent): number => {
  switch (event.deltaMode) {
    case WheelEvent.DOM_DELTA_PIXEL:
      console.log(`wheel_event { delta: ${event.deltaY} }`)
      break
    case WheelEvent.DOM_DELTA_LINE:
      console.error("unhandled page wheel line mode")
      break
    case WheelEvent.DOM_DELTA_PAGE:
      console.error("unhandled page wheel delta mode")
      break
  }
  return event.deltaY
}
