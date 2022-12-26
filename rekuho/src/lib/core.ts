import { writable } from "svelte/store"

export type CaretPosition = { line: number; col: number }

export type State = {
  document_id: string | null
  view_id: string | null
  lines: string[]
  first_line: number
  height: number
  carets: CaretPosition[]
}

export const state = writable<State>({
  document_id: null,
  view_id: null,
  lines: [""],
  first_line: 0,
  height: 10,
  carets: [],
})

export const updateState = <K extends keyof State>(field: K, value: State[K]) => {
  state.update((s) => ({ ...s, [field]: value }))
}
