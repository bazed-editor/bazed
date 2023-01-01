import { v4 as generateUuid } from "uuid"
import { ensureExhaustive } from "./common"
import { state, updateState, type CaretPosition, type State } from "./core"

export const initSession = async () => {
  const websocket = new WebSocket("ws://localhost:6969")
  await new Promise((resolve) => {
    websocket.onopen = (event) => resolve(event)
  })
  return new Session(websocket)
}

/** Session class, holding the session state and the websocket connection to the backend.  */
export class Session {
  websocket: WebSocket
  state: State = {} as any

  constructor(websocket: WebSocket) {
    this.websocket = websocket
    websocket.onmessage = (event) => this.onMessageReceived(JSON.parse(event.data))
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
  handleKeyPressed(input: KeyInput) {
    const view_id = this.state.view_id
    if (view_id) {
      this.send({ method: "key_pressed", params: { view_id, input } })
    }
  }

  /**
   * handle mouse clicks
   * @param {CaretPosition} position - location - as line:column - of click
   */
  handleMouseClicked(position: CaretPosition) {
    const view_id = this.state.view_id
    if (view_id) {
      this.send({ method: "mouse_input", params: { view_id, position } })
    }
  }

  /**
   * handle resizing and view-movements
   * @param {{
   *   height: number // line count
   *   width: number  // column count
   *   first_line: number
   *   first_col: number
   * }} args - object holding new line count, column count, and respective offsets
   */
  handleUpdateView(args: { height: number; width: number; first_line: number; first_col: number }) {
    const view_id = this.state.view_id
    if (view_id) {
      this.send({ method: "viewport_changed", params: { view_id, ...args } })
    }
  }

  /**
   * handles all messages recieved by the frontend, sent by the backend via the established
   * websocket
   */
  async onMessageReceived(message: ToFrontend) {
    console.log("Received message from websocket", message)
    switch (message.method) {
      case "open_document":
        this.onOpenDocument(message.params)
        break
      case "view_opened_response":
        this.onViewOpenedResponse(message.params)
        break
      case "update_view":
        this.onUpdateView(message.params)
        break
      default:
        ensureExhaustive(message)
    }
  }

  /** expected behavior is for a new view to be opened */
  async onViewOpenedResponse(params: ViewOpenedResponse["params"]) {
    updateState("view_id", params.view_id)
  }

  /** expected behavior is for the frontend to update the view */
  async onUpdateView(params: UpdateView["params"]) {
    state.update((state) => ({
      ...state,
      lines: params.text,
      first_line: params.first_line,
      carets: params.carets,
      height: params.height,
    }))
  }

  /** expected behavior is for the frontend to update the viewed document */
  async onOpenDocument(params: OpenDocument["params"]) {
    updateState("document_id", params.document_id)
    const msg: ViewOpened = {
      method: "view_opened",
      params: {
        request_id: generateUuid(),
        document_id: params.document_id,
        height: 200,
        width: 40,
      },
    }
    this.send(msg)
  }
}

type Uuid = string
type RequestId = string

type Position = {
  line: number
  col: number
}

type Message<Method extends string, Params> = {
  method: Method
  params: Params
}

type ToFrontend = OpenDocument | UpdateView | ViewOpenedResponse

type ToBackend = ViewOpened | ViewportChanged | KeyPressed | SaveDocument | MouseInput

type OpenDocument = Message<
  "open_document",
  {
    document_id: Uuid
    path: string | null
    text: string
  }
>

type UpdateView = Message<
  "update_view",
  {
    view_id: Uuid
    first_line: number
    height: number
    text: string[]
    carets: Position[]
  }
>

type ViewOpenedResponse = Message<
  "view_opened_response",
  {
    request_id: RequestId
    view_id: Uuid
  }
>

type ViewOpened = Message<
  "view_opened",
  {
    request_id: RequestId
    document_id: Uuid
    height: number
    width: number
  }
>

type ViewportChanged = Message<
  "viewport_changed",
  {
    view_id: Uuid
    height: number
    width: number
    first_line: number
    first_col: number
  }
>

type KeyPressed = Message<
  "key_pressed",
  {
    view_id: Uuid
    input: KeyInput
  }
>

type SaveDocument = Message<
  "save_document",
  {
    document_id: Uuid
  }
>

// backend unimplemented
type MouseInput = Message<
  "mouse_input",
  {
    view_id: Uuid
    position: Position
  }
>

export type KeyInput = {
  modifiers: Array<Modifier>
  key: Key
}

export type Modifier = "ctrl" | "alt" | "shift" | "win"
export type Key = { char: string } | NonCharKey
export type NonCharKey =
  | "f1"
  | "f2"
  | "f3"
  | "f4"
  | "f5"
  | "f6"
  | "f7"
  | "f8"
  | "f9"
  | "f10"
  | "f11"
  | "f12"
  | "backspace"
  | "return"
  | "tab"
  | "home"
  | "end"
  | "insert"
  | "delete"
  | "page_up"
  | "page_down"
  | "escape"
  | "left"
  | "right"
  | "up"
  | "down"
