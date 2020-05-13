import { H256 } from "..";

/**
 * @hidden
 */
const ALPHABET = "0123456789bcdefghjkmnpqrstuvwxyz";

// FIXME: any
/**
 * @hidden
 */
function convert(data: number[], inBits: number, outBits: number): number[] {
    let value = 0;
    let bits = 0;
    const maxV = (1 << outBits) - 1;

    const result = [];
    for (const datum of data) {
        value = (value << inBits) | datum;
        bits += inBits;

        while (bits >= outBits) {
            bits -= outBits;
            result.push((value >> bits) & maxV);
        }
    }

    if (bits > 0) {
        result.push((value << (outBits - bits)) & maxV);
    }

    return result;
}

// FIXME: any
/**
 * @hidden
 */
export function calculate(
    pubkey: H256,
    networkId: string,
    version: number
): string {
    const bytes = [0, 0, 0, 0, 0];
    const pubkeyHex = Buffer.from(pubkey.value, "hex");
    for (let i = 0; i < 6; i += 1) {
        for (let j = 0; j < 5; j += 1) {
            bytes[j] ^= pubkeyHex[i * 5 + j];
        }
    }

    bytes[3] ^= pubkeyHex[30];
    bytes[4] ^= pubkeyHex[31];

    bytes[0] ^= networkId.charCodeAt(0);
    bytes[1] ^= networkId.charCodeAt(1);
    bytes[2] ^= version;

    const rearranged = convert(bytes, 8, 5);
    return "".concat(...rearranged.map(code => ALPHABET.charAt(code)));
}
