const blake = require("blakejs");
const EC = require("elliptic").ec;

const toHexByte = byte => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16);

const toHex = (buffer: Buffer): string => {
    return Array.from(buffer).map(toHexByte).join("");
};

export const blake256 = (buffer: Buffer): string => {
    const context = blake.blake2bInit(32, null);
    blake.blake2bUpdate(context, buffer);
    return toHex(blake.blake2bFinal(context));
};

export const signEcdsa = (() => {
    const ec = new EC("secp256k1");
    return (message: string, priv: string) => {
        const key = ec.keyFromPrivate(priv);
        return key.sign(message, { "canonical": true });
    };
})();
