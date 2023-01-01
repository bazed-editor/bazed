export type Vector2 = [number, number]

export const merge = (a: Vector2, b: [number | null, number | null]): Vector2 => [
  b[0] === null ? a[0] : b[0],
  b[1] === null ? a[1] : b[1],
]

export const add = (a: Vector2, b: Vector2): Vector2 => [a[0] + b[0], a[1] + b[1]]
export const sub = (a: Vector2, b: Vector2): Vector2 => [a[0] - b[0], a[1] - b[1]]
export const mul = (a: Vector2, b: number): Vector2 => [a[0] * b, a[1] * b]
export const det = (a: Vector2, b: Vector2): number => a[1] * b[0] - a[0] * b[1]
export const floor = (a: Vector2): Vector2 => [Math.floor(a[0]), Math.floor(a[1])]

/** https://en.wikipedia.org/wiki/Cramer%27s_rule */
export const cramer = (a: Vector2, b: Vector2, r: Vector2): Vector2 => [
  det(mul(a, -1), r) / det(a, b),
  det(b, r) / det(a, b),
]
