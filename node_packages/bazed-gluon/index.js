import * as Gluon from "@gluon-framework/gluon"

import { spawn } from "child_process"

import { createServer } from "http"
import { readFile } from "fs/promises"

// To get current directory with node esm
import { join, dirname } from "path"
import { fileURLToPath } from "url"

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

// Start local server for backend
const serverBinaryPath = join(
  __dirname,
  "..",
  "..",
  "target",
  "debug",
  "bazed" + (process.platform === "win32" ? ".exe" : ""),
)
const serverProc = spawn(serverBinaryPath, ["--no-frontend"], {
  stdio: "inherit",
})

// Start local server for frontend (shouldn't be needed in future)
const rekuhoBuildPath = join(__dirname, "..", "bazed-svelte", "build")

const getContentType = (ext) => {
  if (["css", "html"].includes(ext)) return "text/" + ext
  if (ext === "js") return "text/javascript"

  return "text/plain"
}

const httpPort = 9999
const server = createServer(async (req, res) => {
  let reqPath = decodeURI(req.url).split("?").shift().replaceAll("..", "")
  if (reqPath === "/") reqPath = "index.html"

  const path = join(rekuhoBuildPath, reqPath)

  console.log(req.url, path)

  const ext = path.split(".").pop()

  try {
    const content = await readFile(path, "utf8")

    res.writeHead(200, {
      "Content-Type": getContentType(ext),
    })

    res.end(content, "utf8")
  } catch {
    res.writeHead(400)
    res.end("File read error", "utf8")
  }
})

server.listen(httpPort, "localhost", () => {
  console.log("listening on", httpPort)
})

const Window = await Gluon.open(`http://localhost:${httpPort}`)

await Window.page.loaded // wait for load
await new Promise((res) => setTimeout(res, 2000))
Window.page.title = "Bazed (Gluon edition)"
