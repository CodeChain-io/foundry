import { H160 } from "../H160";

test("rlpBytes", () => {
    expect(
        new H160("0000000000000000000000000000000000000000").rlpBytes()
    ).toEqual(
        Buffer.from([
            0x80 + 20,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0
        ])
    );
});
