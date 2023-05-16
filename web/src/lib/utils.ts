export const get_hex = (n: bigint | number | string, len = 10) => Number(n).toString(16).padStart(len, '0');
