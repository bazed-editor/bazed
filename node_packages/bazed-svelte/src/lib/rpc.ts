import { ensureExhaustive } from "./common"
import * as log from "./log"
import { state, type Coordinate, type CoordinateRegion, type State, type Uuid } from "./core"

export const initSession = async (): Promise<Session> => {
  const websocket = new WebSocket("ws://localhost:6969")
  await new Promise((resolve, reject) => {
    websocket.onopen = (event) => resolve(event)
    // websockets will call `Websocket.onerror` then `Websocket.onclose` on failure
    websocket.onclose = (error) => reject(error)
  })
  websocket.onclose = null
  return new Session(websocket)
}

/** Session class, holding the session state and the websocket connection to the backend.  */
export class Session {
  websocket: WebSocket
  state: State = {} as any

  constructor(websocket: WebSocket) {
    websocket.onmessage = (event) => this.onMessageReceived(JSON.parse(event.data))
    this.websocket = websocket
    state.subscribe((state) => {
      this.state = state
    })
  }

  /**
   *
   * @param {ToBackend} message - to send to backend
   */
  send(message: ToBackend) {
    this.websocket.send(JSON.stringify(message))
  }

  /**
   * handle mouse clicks
   * @param {KeyInput} input - key input with active modifiers
   */
  handleKeyPressed(view_id: string, input: KeyInput) {
    this.send({ method: "key_pressed", params: { view_id, input } })
  }

  /**
   * handle mouse clicks
   * @param {CoordinateRegion} position - location - as line:column - of click
   */
  handleMouseClicked(view_id: string, position: Coordinate) {
    this.send({ method: "mouse_input", params: { view_id, position } })
  }

  /**
   * handle mouse wheel
   * @param {number} mouseWheel - count of lines to scroll, positive to scroll down
   */
  handleMouseWheel(view_id: string, mouseWheel: MouseWheel) {
    this.send({ method: "mouse_scroll", params: { view_id, line_delta: mouseWheel.delta } })
  }

  /**
   * handle resizing and view-movements
   * @param {{
   *   height: number // line count
   *   width: number  // column count
   * }} args - object holding new line count, column count, and respective offsets
   */
  handleUpdateView(view_id: string, args: { height: number; width: number }) {
    this.send({ method: "viewport_changed", params: { view_id, ...args } })
  }

  /**
   * handles all messages recieved by the frontend, sent by the backend via the established
   * websocket
   */
  async onMessageReceived(message: ToFrontend) {
    switch (message.method) {
      case "open_view":
        this.onOpenView(message.params)
        break
      case "update_view":
        this.onUpdateView(message.params)
        break
      default:
        ensureExhaustive(message)
    }
  }

  /** expected behavior is for the frontend to update the viewed document */
  async onOpenView(params: OpenView["params"]) {
    state.update((state) => {
      state.views[params.view_id] = {
        filePath: params.path,
        firstLine: params.view_data.first_line,
        lines: params.view_data.text,
        carets: params.view_data.carets,
      }
      return state
    })
  }

  /** expected behavior is for the frontend to update the view */
  async onUpdateView(params: UpdateView["params"]) {
    state.update((state) => {
      const old = state.views[params.view_id]
      if (!old) {
        console.error("Got UpdateView for unknown view id, ignoring...")
      } else {
        state.views[params.view_id] = {
          ...old,
          firstLine: params.view_data.first_line,
          lines: params.view_data.text,
          carets: params.view_data.carets,
        }
      }
      return state
    })
  }
}

type Position = {
  line: number
  col: number
}

type Message<Method extends string, Params> = {
  method: Method
  params: Params
}

type ToFrontend = OpenView | UpdateView

type ToBackend = ViewportChanged | KeyPressed | MouseInput | MouseScroll
type ViewData = {
  first_line: number
  text: string[]
  styles: [CoordinateRegion, TextStyle][]
  carets: CoordinateRegion[]
  vim_mode: string
}

type OpenView = Message<
  "open_view",
  {
    view_id: Uuid
    path: string | null
    view_data: ViewData
  }
>

type UpdateView = Message<
  "update_view",
  {
    view_id: Uuid
    view_data: ViewData
  }
>

type ViewportChanged = Message<
  "viewport_changed",
  {
    view_id: Uuid
    height: number
    width: number
  }
>

type KeyPressed = Message<
  "key_pressed",
  {
    view_id: Uuid
    input: KeyInput
  }
>

type MouseInput = Message<
  "mouse_input",
  {
    view_id: Uuid
    position: Position
  }
>

type MouseScroll = Message<
  "mouse_scroll",
  {
    view_id: Uuid
    line_delta: number
  }
>

export type RgbaColor = [number, number, number, number]

export type Underline = {
  kind: "squiggly" | "zig_zag" | "line" | "dotted"
  color: RgbaColor
}

export type FontStyle = {
  bold: boolean
  italic: boolean
  underline: Underline | null
}

export type TextStyle = {
  foreground: RgbaColor
  background: RgbaColor
  font_style: FontStyle
}

export type MouseWheel = { modifiers: Modifiers; delta: number }

export type KeyInput = {
  modifiers: Modifiers
  key: Key
  code: string
}

export type Modifiers = number
export type Key = string
