/**
 * ```
 * const arr = [0, 0, 0]
 * arrayUpdate(arr, 1, x => x + 1)
 * arr => [0, 1, 0]
 * ```
 */
export const arrayUpdate = <T>(arr: T[], n: number, f: (x: T) => T): void => {
  arr[n] = f(arr[n])
}

/**
 * ```
 * const str = "abcccfg"
 * stringSplice("abcccfg", 4, "de", 2) => "cc"
 * str => "abcdefg"
 * ```
 */
export const stringSplice = (
  str: string,
  offset: number,
  text: string,
  deleteCount: number = 0,
): string => {
  const no = offset < 0 ? self.length + offset : offset
  const nod = no - deleteCount >= 0 ? no - deleteCount : no
  return str.substring(0, nod) + text + str.substring(no)
}

/** Allows to ensure exhaustive matches in switch-case statements */
/* eslint-disable-next-line @typescript-eslint/no-empty-function */
export const ensureExhaustive = (_: never) => {}

/** elko really hates this name */
export const elt = (
  tag: string,
  content?: string | HTMLElement[] | null,
  className?: string | null,
  cssText?: string,
): HTMLElement => {
  const element = document.createElement(tag)
  if (className) {
    element.className = className
  }
  if (cssText) {
    element.style.cssText = cssText
  }
  if (typeof content === "string") {
    element.appendChild(document.createTextNode(content))
  } else if (Array.isArray(content)) {
    for (let i = 0; i < content.length; ++i) {
      element.appendChild(content[i])
    }
  }
  return element
}

export const removeChildrenAndAdd = (
  parent: HTMLElement,
  element: HTMLElement | HTMLElement[],
): HTMLElement => {
  const removeChildren = (element: HTMLElement): HTMLElement => {
    while (element.firstChild) {
      element.removeChild(element.firstChild)
    }
    return element
  }

  if (Array.isArray(element)) {
    removeChildren(parent)
    element.forEach((child) => parent.appendChild(child))
    return parent
  } else {
    return removeChildren(parent).appendChild(element)
  }
}
