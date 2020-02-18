import { Script } from "../Script";

const { NOP, PUSH, PUSHB, CHKTIMELOCK } = Script.Opcode;

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

test("toToken() CHECKTIMELOCK", () => {
    expect(() => {
        new Script(Buffer.from([CHKTIMELOCK, 0])).toTokens();
    }).toThrow("0 is an invalid timelock type");

    const block = new Script(Buffer.from([CHKTIMELOCK, 1])).toTokens();
    expect(block).toEqual(["CHKTIMELOCK", "BLOCK"]);

    const blockAge = new Script(Buffer.from([CHKTIMELOCK, 2])).toTokens();
    expect(blockAge).toEqual(["CHKTIMELOCK", "BLOCK_AGE"]);

    const time = new Script(Buffer.from([CHKTIMELOCK, 3])).toTokens();
    expect(time).toEqual(["CHKTIMELOCK", "TIME"]);

    const timeAge = new Script(Buffer.from([CHKTIMELOCK, 4])).toTokens();
    expect(timeAge).toEqual(["CHKTIMELOCK", "TIME_AGE"]);

    expect(() => {
        new Script(Buffer.from([CHKTIMELOCK, 5])).toTokens();
    }).toThrow("5 is an invalid timelock type");
});
