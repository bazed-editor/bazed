import { v4 as generateUuid } from "uuid"
import { ensureExhaustive } from "./common"
import { state, type CaretPosition, type State, type ViewState } from "./core"
import * as log from "./log"

export const initSession = async (attempts?: number): Promise<Session> => {
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
  requests: { [id: string]: (response: ToFrontend) => void }

  constructor(websocket: WebSocket) {
    websocket.onmessage = (event) => this.onMessageReceived(JSON.parse(event.data))
    this.websocket = websocket
    this.requests = {}
    state.subscribe((state) => {
      this.state = state
    })
  }

  /**
   *
   * @param {ToBackend} message - to send to backend
   */
  send(message: ToBackend) {
    log.debug("Dispatching rpc to backend:", message)
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
   * @param {CaretPosition} position - location - as line:column - of click
   */
  handleMouseClicked(view_id: string, position: CaretPosition) {
    this.send({ method: "mouse_input", params: { view_id, position } })
  }

  /**
   * handle mouse wheel
   * @param {number} line_delta - count of lines to scroll, positive to scroll down
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
    log.info("Message received via websocket:", message)
    switch (message.method) {
      case "open_document":
        this.onOpenDocument(message.params)
        break
      case "update_view":
        this.onUpdateView(message.params)
        break
      case "view_opened_response":
        this.requests[message.params.request_id](message)
        delete this.requests[message.params.request_id]
        break
      default:
        ensureExhaustive(message)
    }
  }

  async requestDocumentView(document_id: string): Promise<ViewOpenedResponse> {
    const request_id = generateUuid()

    const message: ViewOpened = {
      method: "view_opened",
      params: {
        document_id,
        request_id,
        height: 200,
        width: 40,
      },
    }

    this.send(message)
    return new Promise((resolve) => (this.requests[request_id] = resolve as any))
  }

  /** expected behavior is for the frontend to update the viewed document */
  async onOpenDocument(params: OpenDocument["params"]) {
    const document: { path: string | null } = { path: params.path }

    state.update((state) => ({
      views: state.views,
      documents: {
        ...state.documents,
        [params.document_id]: document,
      },
    }))

    let viewOpenResponse = await this.requestDocumentView(params.document_id)
    this.onViewOpenedResponse(params.document_id, viewOpenResponse.params)
  }

  /** expected behavior is for a new view to be opened */
  async onViewOpenedResponse(document_id: string, params: ViewOpenedResponse["params"]) {
    const normal: ViewState = {
      document: document_id,
      lines: [],
      firstLine: 0,
      height: 20,
      carets: [],
    }

    state.update((state) => ({
      documents: state.documents,
      views: {
        ...state.views,
        [params.view_id]: normal,
      },
    }))
  }

  /** expected behavior is for the frontend to update the view */
  async onUpdateView(params: UpdateView["params"]) {
    state.update((state) => {
      state.views[params.view_id] = {
        ...state.views[params.view_id],
        firstLine: params.first_line,
        lines: params.text,
        carets: params.carets,
      }
      return state
    })
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

type ToBackend = ViewOpened | ViewportChanged | KeyPressed | SaveDocument | MouseInput | MouseScroll

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

type MouseScroll = Message<
  "mouse_scroll",
  {
    view_id: Uuid
    line_delta: number
  }
>

export type MouseWheel = { modifiers: Modifiers; delta: number }

export type KeyInput = {
  modifiers: Modifiers
  key: Key
  code: string
}

export type Modifiers = number
export type Key = string
