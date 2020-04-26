import BigNumber from "bignumber.js";
import {
    generatePrivateKey,
    getAccountIdFromPrivate,
    getAccountIdFromPublic,
    getPublicFromPrivate,
    toHex,
    toLocaleString
} from "..";

test.each([
    [[0x00, 0x01, 0x02], "000102"],
    [[0xff, 0xfe, 0xfd], "fffefd"],
    [[0xde, 0xad, 0xbe, 0xef], "deadbeef"],
    [[0x62, 0x75, 0x66, 0x66, 0x65, 0x72], "627566666572"]
])("toHex %p", (input, output) => {
    const buffer = new Buffer(input);
    expect(toHex(buffer)).toEqual(output);
});

test("getAccountIdFromPrivate", () => {
    const priv = generatePrivateKey();
    const accountId = getAccountIdFromPrivate(priv);
    expect(/^[0-9a-fA-F]{40}$/.test(accountId)).toBe(true);
});

test("getAccountIdFromPublic", () => {
    const priv = generatePrivateKey();
    const pub = getPublicFromPrivate(priv);
    const accountId = getAccountIdFromPublic(pub);
    expect(/^[0-9a-fA-F]{40}$/.test(accountId)).toBe(true);
});

test("toLocaleString", () => {
    expect(toLocaleString(new BigNumber(1234567))).toBe("1,234,567");
    expect(toLocaleString(new BigNumber(123))).toBe("123");
    expect(
        toLocaleString(new BigNumber("1234123412341234.1234123412341234"))
    ).toBe("1,234,123,412,341,234.1234123412341234");
    expect(toLocaleString(new BigNumber("-1234234.234134234"))).toBe(
        "-1,234,234.234134234"
    );
});
