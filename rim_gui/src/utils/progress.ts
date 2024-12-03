export function progressFormat(value: number) {
  return value.toFixed(2).padStart(5, '0') + '%';
}
