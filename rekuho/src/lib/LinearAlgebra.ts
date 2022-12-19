export type Vector = [number, number]

export const merge = (a: Vector, b: [number | null, number | null]): Vector => [
  b[0] === null ? a[0] : b[0],
  b[1] === null ? a[1] : b[1],
]

export const add = (a: Vector, b: Vector): Vector => [a[0] + b[0], a[1] + b[1]]
export const sub = (a: Vector, b: Vector): Vector => [a[0] - b[0], a[1] - b[1]]
export const mul = (a: Vector, b: number): Vector => [a[0] * b, a[1] * b]
export const det = (a: Vector, b: Vector): number => a[1] * b[0] - a[0] * b[1]

export const cramer = (a: Vector, b: Vector, r: Vector): Vector => [
  det(mul(a, -1), r) / det(a, b),
  det(b, r) / det(a, b),
]
