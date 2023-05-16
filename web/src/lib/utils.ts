export const get_hex = (n: bigint | number, len = 10) => n.toString(16).padStart(len, '0');
