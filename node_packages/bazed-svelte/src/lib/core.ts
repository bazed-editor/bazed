import { writable } from "svelte/store"

export type Uuid = string
export type ViewId = Uuid
export type Coordinate = { line: number; col: number }
export type CoordinateRegion = { head: Coordinate; tail: Coordinate }

/** cached view state from backend */
export type State = {
  views: {
    [id: ViewId]: {
      filePath: string | null
      lines: string[]
      firstLine: number
      carets: CoordinateRegion[]
    }
  }
}

/** store, holding cached state from backend */
export const state = writable<State>({ views: {} })
