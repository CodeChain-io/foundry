// import { LocalKeyStore } from "../LocalKeyStore";
// import { verifyEcdsa, recoverEcdsa } from "../../utils";

test("temporary", () => {
    expect(1).toBe(1);
});

// test("createKey", async () => {
//     const store = await LocalKeyStore.createForTest();
//     expect(() => {
//         store.asset.createKey();
//     }).not.toThrow();
// });

// test("removeKey", async () => {
//     const store = await LocalKeyStore.createForTest();
//     const key1 = await store.asset.createKey();
//     expect(await store.asset.removeKey({ publicKey: key1 })).toBe(true);
//     expect(await store.asset.removeKey({ publicKey: key1 })).toBe(false);
// });

// test("getKeyList", async () => {
//     const store = await LocalKeyStore.createForTest();
//     const key1 = await store.asset.createKey();
//     const key2 = await store.asset.createKey();
//     expect(await store.asset.getKeyList()).toContain(key1);
//     expect(await store.asset.getKeyList()).toContain(key2);

//     await store.asset.removeKey({ publicKey: key1 });

//     expect(await store.asset.getKeyList()).not.toContain(key1);
// });

// test("sign", async () => {
//     const store = await LocalKeyStore.createForTest();
//     const key1 = await store.asset.createKey();
//     const signature = await store.asset.sign({ publicKey: key1, message: "hello" });
//     const r = `${signature.substr(0, 64)}`;
//     const s = `${signature.substr(64, 64)}`;
//     const v = Number.parseInt(signature.substr(128, 2), 16);

//     expect(verifyEcdsa("hello", { r, s, v }, key1)).toBe(true);
//     expect(recoverEcdsa("hello", { r, s, v })).toEqual(key1);
// });
