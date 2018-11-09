"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const buffer_1 = require("buffer");
const _ = require("lodash");
class Script {
    constructor(data) {
        this.data = buffer_1.Buffer.from(data);
    }
    /**
     * Creates empty script.
     * @returns Script
     */
    static empty() {
        return new Script(buffer_1.Buffer.from([]));
    }
    /**
     * Converts script to string tokens.
     * @returns Array of string. Each string is a Opcode name or hexadecimal
     *          string for a value
     * @throws When unknown opcode exists in the script
     * @throws When the parameter is expected but not exists
     */
    toTokens() {
        const { data } = this;
        const tokens = [];
        const { NOP, BURN, NOT, EQ, JMP, JNZ, JZ, PUSH, POP, PUSHB, DUP, SWAP, COPY, DROP, CHKSIG, BLAKE256, SHA256, RIPEMD160, KECCAK256, BLAKE160 } = Script.Opcode;
        let cursor = 0;
        while (cursor < data.length) {
            const opcode = data.readUInt8(cursor++);
            const name = _.invert(Script.Opcode)[opcode];
            switch (opcode) {
                case NOP:
                case BURN:
                case NOT:
                case EQ:
                case POP:
                case DUP:
                case SWAP:
                case CHKSIG:
                case BLAKE256:
                case SHA256:
                case RIPEMD160:
                case KECCAK256:
                case BLAKE160:
                    tokens.push(name);
                    break;
                case PUSHB:
                    if (data.length < cursor + 1) {
                        throw Error(`The parameter of ${name} is expected but not exists`);
                    }
                    const len = data.readUInt8(cursor++);
                    if (data.length < cursor + len) {
                        throw Error(`The parameter of ${name} is expected but not exists`);
                    }
                    const blob = data.subarray(cursor, cursor + len);
                    cursor += len;
                    tokens.push(name);
                    tokens.push(`0x${buffer_1.Buffer.from(Array.from(blob))
                        .toString("hex")
                        .toUpperCase()}`);
                    break;
                case PUSH:
                case JMP:
                case JNZ:
                case JZ:
                case COPY:
                case DROP:
                    let val;
                    try {
                        val = data.readUInt8(cursor++);
                    }
                    catch (_) {
                        throw Error(`The parameter of ${name} is expected but not exists`);
                    }
                    tokens.push(name);
                    tokens.push(`0x${val.toString(16).toUpperCase()}`);
                    break;
                default:
                    throw Error(`Unknown opcode: 0x${opcode.toString(16).toUpperCase()}`);
            }
        }
        return tokens;
    }
}
Script.Opcode = {
    NOP: 0x00,
    BURN: 0x01,
    NOT: 0x10,
    EQ: 0x11,
    JMP: 0x20,
    JNZ: 0x21,
    JZ: 0x22,
    PUSH: 0x30,
    POP: 0x31,
    PUSHB: 0x32,
    DUP: 0x33,
    SWAP: 0x34,
    COPY: 0x35,
    DROP: 0x36,
    CHKSIG: 0x80,
    BLAKE256: 0x90,
    SHA256: 0x91,
    RIPEMD160: 0x92,
    KECCAK256: 0x93,
    BLAKE160: 0x94,
    CHKTIMELOCK: 0xb0
};
exports.Script = Script;
