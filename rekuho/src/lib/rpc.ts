import { v4 } from "uuid"
import { state, updateState, type CaretPosition, type State } from "./core"

function ensureExhaustive(_: never) {}

export async function initSession() {
  const ws = new WebSocket("ws://localhost:6969")
  await new Promise((resolve) => {
    ws.onopen = (event) => resolve(event)
  })
  return new Session(ws)
}

export class Session {
  ws: WebSocket
  state: State = {} as any

  constructor(ws: WebSocket) {
    this.ws = ws
    ws.onmessage = (event) => {
      this.onMessageReceived(JSON.parse(event.data))
    }
    state.subscribe((state) => {
      this.state = state
    })
  }

  send(msg: ToBackend) {
    this.ws.send(JSON.stringify(msg))
  }

  async handleKeyPressed(input: KeyInput) {
    const view_id = this.state.view_id
    if (view_id) {
      this.send({ method: "key_pressed", params: { view_id, input } })
    }
  }
  async handleMouseClicked(pos: CaretPosition) {
    const view_id = this.state.view_id
    if (view_id) {
      this.send({ method: "mouse_input", params: { view_id, position: pos } })
    }
  }

  async onMessageReceived(msg: ToFrontend) {
    console.log("Got message from ws", msg)
    switch (msg.method) {
      case "open_document":
        this.onOpenDocument(msg.params)
        break
      case "view_opened_response":
        this.onViewOpenedResponse(msg.params)
        break
      case "update_view":
        this.onUpdateView(msg.params)
        break
      default:
        ensureExhaustive(msg)
    }
  }

  async onUpdateView(params: UpdateView["params"]) {
    state.update((state) => ({
      ...state,
      lines: params.text,
      first_line: params.first_line,
      carets: params.carets,
      height: params.height,
    }))
  }

  async onOpenDocument(params: OpenDocument["params"]) {
    updateState("document_id", params.document_id)
    const msg: ViewOpened = {
      method: "view_opened",
      params: {
        request_id: v4(),
        document_id: params.document_id,
        height: 20,
        width: 40,
      },
    }
    this.send(msg)
  }

  async onViewOpenedResponse(params: ViewOpenedResponse["params"]) {
    updateState("view_id", params.view_id)
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
