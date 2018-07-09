import { encode, decode, toWords } from "../bech32";

test("encode platform account address version 0", () => {
    // words: 0, 1, 2, 3, 4, 5, 6, 7
    const encoded = encode("ccc", toWords([0x00, 0x44, 0x32, 0x14, 0xc7]));
    expect(encoded).toEqual("cccqpzry9x848mh92");
});

test("decode platform account address version 0", () => {
    const { prefix, words } = decode("cccqpzry9x848mh92", "ccc");

    expect(prefix).toEqual("ccc");
    expect(words).toEqual([0, 1, 2, 3, 4, 5, 6, 7]);
});
