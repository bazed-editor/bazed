// const arr = [0, 0, 0]
// arrayUpdate(arr, 1, x => x + 1)
// arr => [0, 1, 0]
export const arrayUpdate = <T>(arr: T[], n: number, f: (x: T) => T): void => {
  arr[n] = f(arr[n])
}

// const str = "abcccfg"
// stringSplice("abcccfg", 4, "de", 2) => "cc"
// str => "abcdefg"
export const stringSplice = (
  str: string,
  offset: number,
  text: string,
  deleteCount: number = 0,
): string => {
  const no = offset < 0 ? self.length + offset : offset
  const nod = no - deleteCount >= 0 ? no - deleteCount : no
  return str.substring(0, nod) + text + str.substring(no)
}
