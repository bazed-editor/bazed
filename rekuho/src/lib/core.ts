import { writable } from "svelte/store"

export type CaretPosition = { line: number; col: number }

/** cached view state from backend */
export type State = {
  documentId: string | null
  viewId: string | null
  lines: string[]
  firstLine: number
  height: number
  carets: CaretPosition[]
}

/** store, holding cached state from backend */
export const state = writable<State>({
  documentId: null,
  viewId: null,
  lines: [""],
  firstLine: 0,
  height: 10,
  carets: [],
})

/** update stored cached state */
export const updateState = <K extends keyof State>(field: K, value: State[K]) => {
  state.update((current) => ({ ...current, [field]: value }))
}
