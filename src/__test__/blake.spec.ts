import { blake208, blake256 } from "../utils";

test("result of blake256 is 64 hex decimal", () => {
    const hash = blake256("some string");
    expect(hash.length).toBe(64);
});


test("result of blake208 is 64 hex decimal", () => {
    const hash = blake208("some string");
    expect(hash.length).toBe(52);
});
