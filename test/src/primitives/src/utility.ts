import BigNumber from "bignumber.js";

/**
 * @hidden
 */
const toHexByte = (byte: number) =>
    byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16);

/**
 * Converts buffer to hexadecimal string.
 * @param buffer arbritrary length of data
 * @returns hexadecimal string
 */
export const toHex = (buffer: Buffer): string => {
    return Array.from(buffer)
        .map(toHexByte)
        .join("");
};

/**
 * Converts hexadecimal string to Uint8Array.
 * @param string arbritrary length of data
 * @returns Uint8Array
 */
export const toArray = (hex: string): Uint8Array => {
    return Uint8Array.from(Buffer.from(hex, "hex"));
};

/**
 * Converts BigNumber to formatted number string
 * Default decimalSeparator is point: "."
 * Default groupSeparator is comma; ","
 * Default groupSize is 3
 * @param num BigNumber object
 * @returns formatted number string
 */
export const toLocaleString = (num: BigNumber): string => {
    return num.toFormat();
};
