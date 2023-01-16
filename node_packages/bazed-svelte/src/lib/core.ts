import { writable } from "svelte/store"

export type Uuid = string
export type ViewId = Uuid
export type CaretPosition = { line: number; col: number }

/** cached view state from backend */
export type State = {
  views: {
    [id: ViewId]: {
      filePath: string | null
      lines: string[]
      firstLine: number
      carets: CaretPosition[]
    }
  }
}

/** store, holding cached state from backend */
export const state = writable<State>({ views: {} })
