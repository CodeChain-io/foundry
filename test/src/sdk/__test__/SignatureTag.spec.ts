import { expect } from "chai";
import * as _ from "lodash";
import "mocha";
import { encodeSignatureTag } from "../utils";

([
    ["all", "all", [0b00000011]],
    ["all", [], [0b00000001]],
    ["all", [0], [0b00000001, 0b00000101]],
    ["all", [7], [0b10000000, 0b00000101]],
    ["all", [8], [0b00000001, 0b00000000, 0b00001001]],
    ["all", [10], [0b00000100, 0b00000000, 0b00001001]],
    ["all", [0, 1, 2, 3, 4, 5, 6, 7], [0b11111111, 0b00000101]],
    ["all", [0, 1, 2, 3, 4, 5, 6, 7, 8], [0b00000001, 0b11111111, 0b00001001]],
    [
        "all",
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        [0b00000011, 0b11111111, 0b00001001]
    ],
    [
        "all",
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        [0b00000111, 0b11111111, 0b00001001]
    ],
    [
        "all",
        [503],
        [0b10000000, ..._.times(62, _.constant(0b00000000)), 0b11111101]
    ],
    ["all", _.range(504), [..._.times(63, _.constant(0b11111111)), 0b11111101]],
    ["single", "all", [0b00000010]],
    ["single", [], [0b00000000]],
    ["single", [7], [0b10000000, 0b00000100]],
    [
        "single",
        _.range(504), // [0, 1, 2, ..., 503]
        [..._.times(63, _.constant(0b11111111)), 0b11111100]
    ]
] as ["all" | "single", "all" | number[], number[]][]).forEach(
    ([input, output, expected]) => {
        it(`{ input: ${input}, output: ${output} }`, function() {
            expect(encodeSignatureTag({ input, output })).deep.equal(
                Buffer.from(expected)
            );
        });
    }
);

describe("Invalid signature tag", () => {
    it("Out of range", () => {
        expect(() =>
            encodeSignatureTag({ input: "all", output: [0, -1] })
        ).throw("-1");
        expect(() =>
            encodeSignatureTag({ input: "all", output: [0, 504] })
        ).throw("504");
    });

    it("Invalid type", () => {
        expect(() =>
            encodeSignatureTag({
                input: "all",
                output: "invalid_string" as any
            })
        ).throw("invalid_string");
        expect(() =>
            encodeSignatureTag({
                input: "invalid_string" as any,
                output: "all"
            })
        ).throw("invalid_string");
    });

    it("Invalid output", () => {
        expect(() =>
            encodeSignatureTag({ input: "all", output: ["0" as any] })
        ).throw("0");
        expect(() =>
            encodeSignatureTag({ input: "all", output: [null as any] })
        ).throw("null");
    });
});
