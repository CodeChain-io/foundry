/**
 * @hidden
 */
const ALPHABET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/* tslint:disable:prefer-for-of */

// pre-compute lookup table
/**
 * @hidden
 */
const ALPHABET_MAP: { [ch: string]: number } = {};
for (let z = 0; z < ALPHABET.length; z++) {
    const x = ALPHABET.charAt(z);

    if (ALPHABET_MAP[x] !== undefined) {
        throw new TypeError(x + " is ambiguous");
    }
    ALPHABET_MAP[x] = z;
}

// FIXME: any
/**
 * @hidden
 */
function polymodStep(pre: any) {
    const b = pre >> 25;
    return (
        ((pre & 0x1ffffff) << 5) ^
        (-((b >> 0) & 1) & 0x3b6a57b2) ^
        (-((b >> 1) & 1) & 0x26508e6d) ^
        (-((b >> 2) & 1) & 0x1ea119fa) ^
        (-((b >> 3) & 1) & 0x3d4233dd) ^
        (-((b >> 4) & 1) & 0x2a1462b3)
    );
}

// FIXME: any
/**
 * @hidden
 */
function prefixChk(prefix: any) {
    let chk = 1;
    for (let i = 0; i < prefix.length; ++i) {
        const c = prefix.charCodeAt(i);
        if (c < 33 || c > 126) {
            throw new Error("Invalid prefix (" + prefix + ")");
        }

        chk = polymodStep(chk) ^ (c >> 5);
    }
    chk = polymodStep(chk);

    for (let i = 0; i < prefix.length; ++i) {
        const v = prefix.charCodeAt(i);
        chk = polymodStep(chk) ^ (v & 0x1f);
    }
    return chk;
}

// FIXME: any
export function encode(prefix: any, words: any, LIMIT?: any) {
    LIMIT = LIMIT || 90;
    if (prefix.length + 7 + words.length > LIMIT) {
        throw new TypeError("Exceeds length limit");
    }

    prefix = prefix.toLowerCase();

    // determine chk mod
    let chk = prefixChk(prefix);
    let result = prefix;
    for (let i = 0; i < words.length; ++i) {
        const x = words[i];
        if (x >> 5 !== 0) {
            throw new Error("Non 5-bit word");
        }

        chk = polymodStep(chk) ^ x;
        result += ALPHABET.charAt(x);
    }

    for (let i = 0; i < 6; ++i) {
        chk = polymodStep(chk);
    }
    chk ^= 1;

    for (let i = 0; i < 6; ++i) {
        const v = (chk >> ((5 - i) * 5)) & 0x1f;
        result += ALPHABET.charAt(v);
    }

    return result;
}

// FIXME: any
export function decode(str: string, prefix: string, LIMIT?: number) {
    LIMIT = LIMIT || 90;
    if (str.length < 8) {
        throw new TypeError(str + " too short");
    }
    if (str.length > LIMIT) {
        throw new TypeError("Exceeds length limit");
    }

    // don't allow mixed case
    const lowered = str.toLowerCase();
    const uppered = str.toUpperCase();
    if (str !== lowered && str !== uppered) {
        throw new Error("Mixed-case string " + str);
    }
    str = lowered;

    if (!str.startsWith(prefix)) {
        throw new Error("Missing prefix for " + str);
    }
    const split = prefix.length;

    const wordChars = str.slice(split);
    if (wordChars.length < 6) {
        throw new Error("Data too short");
    }

    let chk = prefixChk(prefix);
    const words = [];
    for (let i = 0; i < wordChars.length; ++i) {
        const c = wordChars.charAt(i);
        const v = ALPHABET_MAP[c];
        if (v === undefined) {
            throw new Error("Unknown character " + c);
        }
        chk = polymodStep(chk) ^ v;

        // not in the checksum?
        if (i + 6 >= wordChars.length) {
            continue;
        }
        words.push(v);
    }

    if (chk !== 1) {
        throw new Error("Invalid checksum for " + str);
    }
    return { prefix, words };
}

// FIXME: any
/**
 * @hidden
 */
function convert(data: any, inBits: any, outBits: any, pad: any) {
    let value = 0;
    let bits = 0;
    const maxV = (1 << outBits) - 1;

    const result = [];
    for (let i = 0; i < data.length; ++i) {
        value = (value << inBits) | data[i];
        bits += inBits;

        while (bits >= outBits) {
            bits -= outBits;
            result.push((value >> bits) & maxV);
        }
    }

    if (pad) {
        if (bits > 0) {
            result.push((value << (outBits - bits)) & maxV);
        }
    } else {
        if (bits >= inBits) {
            throw new Error("Excess padding");
        }
        if ((value << (outBits - bits)) & maxV) {
            throw new Error("Non-zero padding");
        }
    }

    return result;
}

// FIXME: any
export function toWords(bytes: any) {
    return convert(bytes, 8, 5, true);
}

// FIXME: any
export function fromWords(words: any) {
    return convert(words, 5, 8, false);
}
