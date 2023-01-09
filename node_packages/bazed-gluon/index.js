import * as Gluon from "@gluon-framework/gluon"

import { spawn } from "child_process"

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

const Window = await Gluon.open(rekuhoBuildPath)

await Window.page.loaded // wait for load
await new Promise((res) => setTimeout(res, 2000))
Window.page.title = "Bazed (Gluon edition)"
