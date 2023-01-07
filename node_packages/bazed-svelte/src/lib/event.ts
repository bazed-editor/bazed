import type { KeyInput, Modifier } from "./rpc"

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
  return { modifiers, key: event.key, code: event.code }
}

export const wheelDelta = (event: WheelEvent): number => {
  switch (event.deltaMode) {
    case WheelEvent.DOM_DELTA_PIXEL:
      console.log(`wheel_event { delta: ${event.deltaY} }`)
      break
    case WheelEvent.DOM_DELTA_LINE:
      console.error('improperly handled wheel delta mode: "LINE"')
      break
    case WheelEvent.DOM_DELTA_PAGE:
      console.error('improperly handled wheel delta mode: "PAGE"')
      break
  }
  return Math.sign(event.deltaY)
}
