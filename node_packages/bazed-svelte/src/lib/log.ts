type Log = (...params: any[]) => void
export type Level = "debug" | "info" | "warn" | "error"

const irrelevance = {
  error: 0,
  warn: 1,
  info: 2,
  debug: 3,
}

const levelColor = {
  debug: "lightblue",
  info: "lightgreen",
  warn: "yellow",
  error: "lightred",
}

type Logger = (level: Level, location?: string, message?: any[], ...params: any[]) => void

const firebug = (level: Level, location?: string, message?: any[], ...params: any[]): void => {
  const time = Date.now()
  let args = [
    `%c${time} %c${level.toUpperCase()}  ${message}`,
    `color: gray`,
    `color: ${levelColor[level]};`,
  ]
  if (location) {
    args[0] = args[0].concat(`\n%cat %c${location}`)
    args = args.concat([`color: gray`, `color: white`])
  }
  console.log(...args, ...params)
}

export const LOGGERS: Logger[] = [firebug]
export let LEVEL: Level = "debug"

const debugInfo = (height: number = 5): string | undefined => {
  const error = new Error()
  const regex = /\((.*):(\d+):(\d+)\)$/
  if (error.stack) {
    const caller = regex.exec(error.stack?.split("\n")[height])
    if (caller) {
      const file = caller[1]
      const line = caller[2]
      const column = caller[3]
      return `${file}:${line}:${column}`
    }
  }
  return undefined
}

const withLocation = (log: (location?: string, ...params: any[]) => void): Log => {
  const location = debugInfo()
  return (...params) => log(location ? location : "unlocated", ...params)
}

export const _log = (level: Level, location?: string, message?: any[], ...params: any[]): void => {
  if (irrelevance[level] <= irrelevance[LEVEL]) {
    for (const log of LOGGERS) {
      log(level, location, message, ...params)
    }
  }
}

export const log = (level: Level, ...params: any[]) =>
  withLocation((location, ...params: any[]) => _log(level, location, ...params))(...params)

export const info = (...params: any[]) => log("info", ...params)
export const debug = (...params: any[]) => log("debug", ...params)
export const warn = (...params: any[]) => log("warn", ...params)
export const error = (...params: any[]) => log("error", ...params)
