import type { KeyInput, Modifiers } from "./rpc"
import * as log from "./log"

const ModifierBits = {
  CTRL: 0b00000001,
  SHIFT: 0b00000010,
  ALT: 0b00000100,
  WIN: 0b00001000,
}

export const getModifiers = (event: WheelEvent | KeyboardEvent): Modifiers => {
  let modifiers: Modifiers = 0
  if (event.ctrlKey) modifiers |= ModifierBits.CTRL
  if (event.shiftKey) modifiers |= ModifierBits.SHIFT
  if (event.altKey) modifiers |= ModifierBits.ALT
  if (event.metaKey) modifiers |= ModifierBits.WIN
  return modifiers
}

export const keyboardToKeyInput = (event: KeyboardEvent): KeyInput | null => {
  const modifiers = getModifiers(event)
  return { modifiers, key: event.key, code: event.code }
}

export const wheelDelta = (event: WheelEvent): number => {
  switch (event.deltaMode) {
    case WheelEvent.DOM_DELTA_PIXEL:
      break
    case WheelEvent.DOM_DELTA_LINE:
      log.warn('improperly handled wheel delta mode: "LINE"')
      break
    case WheelEvent.DOM_DELTA_PAGE:
      log.warn('improperly handled wheel delta mode: "PAGE"')
      break
  }
  return Math.sign(event.deltaY)
}
