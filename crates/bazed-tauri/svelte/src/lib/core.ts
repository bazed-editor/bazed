import { writable } from "svelte/store"

export type CaretPosition = { line: number; col: number }

/** cached view state from backend */
export type State = {
  documents: { [id: string]: { path: string | null } }
  views: { [id: string]: ViewState }
}

export type ViewState = {
  document: string
  lines: string[]
  firstLine: number
  height: number
  carets: CaretPosition[]
}

/** store, holding cached state from backend */
export const state = writable<State>({ documents: {}, views: {} })

/** update stored cached state */
export const updateState = <K extends keyof State>(field: K, value: State[K]) => {
  state.update((current) => ({ ...current, [field]: value }))
}
