import { blake256 } from "../utils";

test("result of blake256 is 64 hex decimal", () => {
    const hash = blake256("some string");
    expect(hash.length).toBe(64);
});
