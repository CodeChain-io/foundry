import { Script } from "../Script";

const {
    NOP,
    BURN,
    NOT,
    EQ,
    JMP,
    JNZ,
    JZ,
    PUSH,
    POP,
    PUSHB,
    DUP,
    SWAP,
    COPY,
    DROP,
    CHKSIG,
    BLAKE256,
    SHA256,
    RIPEMD160,
    KECCAK256
} = Script.Opcode;

test("Script.empty()", () => {
    expect(() => {
        Script.empty();
    }).not.toThrow();
});

test("toTokens()", () => {
    let tokens;

    tokens = new Script(Buffer.from([NOP])).toTokens();
    expect(tokens).toEqual(["NOP"]);

    tokens = new Script(Buffer.from([NOP, NOP])).toTokens();
    expect(tokens).toEqual(["NOP", "NOP"]);

    tokens = new Script(Buffer.from([PUSH, 0xff])).toTokens();
    expect(tokens).toEqual(["PUSH", "0xFF"]);

    tokens = new Script(
        Buffer.from([PUSHB, 3, 0xff, 0xee, 0xdd, NOP])
    ).toTokens();
    expect(tokens).toEqual(["PUSHB", "0xFFEEDD", "NOP"]);
});

test("toToken() throws when unknown opcode is given", () => {
    expect(() => {
        // 0xFF is not an opcode
        new Script(Buffer.from([0xff])).toTokens();
    }).toThrow("Unknown opcode: 0xFF");
});

test("toToken() throws when the parameter is expected but not exists", () => {
    expect(() => {
        new Script(Buffer.from([PUSH])).toTokens();
    }).toThrow("The parameter of PUSH is expected but not exists");

    expect(() => {
        new Script(Buffer.from([PUSHB])).toTokens();
    }).toThrow("The parameter of PUSHB is expected but not exists");

    expect(() => {
        new Script(Buffer.from([PUSHB, 4, 0xaa, 0xbb])).toTokens();
    }).toThrow("The parameter of PUSHB is expected but not exists");

    expect(() => {
        new Script(Buffer.from([PUSHB, 3, 0xaa, 0xbb])).toTokens();
    }).toThrow("The parameter of PUSHB is expected but not exists");
});
