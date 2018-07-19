import { MemoryRawKeyStore } from "../MemoryRawKeyStore";
import { verifyEcdsa, recoverEcdsa } from "../../utils";

test("createKey", async () => {
    const store = new MemoryRawKeyStore();
    expect(() => {
        store.createKey();
    }).not.toThrow();
});

test("removeKey", async () => {
    const store = new MemoryRawKeyStore();
    const key1 = await store.createKey();
    expect(await store.removeKey({ publicKey: key1 })).toBe(true);
    expect(await store.removeKey({ publicKey: key1 })).toBe(false);
});

test("getKeyList", async () => {
    const store = new MemoryRawKeyStore();
    const key1 = await store.createKey();
    const key2 = await store.createKey();
    expect(await store.getKeyList()).toContain(key1);
    expect(await store.getKeyList()).toContain(key2);

    await store.removeKey({ publicKey: key1 });

    expect(await store.getKeyList()).not.toContain(key1);
});

test("sign", async () => {
    const store = new MemoryRawKeyStore();
    const key1 = await store.createKey();
    const signature = await store.sign({ publicKey: key1, message: "hello" });

    expect(verifyEcdsa("hello", signature, key1)).toBe(true);
    expect(recoverEcdsa("hello", signature)).toEqual(key1);
});
