import BigNumber from "bignumber.js";
import {
    generatePrivateKey,
    getPublicFromPrivate,
    toHex,
    toLocaleString
} from "../src";
import "mocha";
import { expect } from "chai";

([
    [[0x00, 0x01, 0x02], "000102"],
    [[0xff, 0xfe, 0xfd], "fffefd"],
    [[0xde, 0xad, 0xbe, 0xef], "deadbeef"],
    [[0x62, 0x75, 0x66, 0x66, 0x65, 0x72], "627566666572"]
] as [number[], string][]).forEach(args => {
    const [input, output] = args;
    it(`toHex ${input} ${output}`, () => {
        const buffer = new Buffer(input);
        expect(toHex(buffer)).equal(output);
    });
});

it("getPublicFromPrivate", () => {
    const priv = generatePrivateKey();
    const pubkey = getPublicFromPrivate(priv);
    expect(/^[0-9a-fA-F]{64}$/.test(pubkey)).true;
});

it("toLocaleString", () => {
    expect(toLocaleString(new BigNumber(1234567))).equal("1,234,567");
    expect(toLocaleString(new BigNumber(123))).equal("123");
    expect(
        toLocaleString(new BigNumber("1234123412341234.1234123412341234"))
    ).equal("1,234,123,412,341,234.1234123412341234");
    expect(toLocaleString(new BigNumber("-1234234.234134234"))).equal(
        "-1,234,234.234134234"
    );
});
