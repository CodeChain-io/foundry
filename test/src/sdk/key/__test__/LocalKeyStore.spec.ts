import {
    getAccountIdFromPublic,
    getPublicFromPrivate,
    verifyEd25519
} from "../../utils";
import { LocalKeyStore } from "../LocalKeyStore";

test("createKey", async () => {
    const store = await LocalKeyStore.createForTest();
    await expect(store.createKey()).resolves.toEqual(expect.anything());
});

test("removeKey", async () => {
    const store = await LocalKeyStore.createForTest();
    const key1 = await store.createKey();
    expect(await store.removeKey({ key: key1 })).toBe(true);
    expect(await store.removeKey({ key: key1 })).toBe(false);
});

test("getKeyList", async () => {
    const store = await LocalKeyStore.createForTest();
    const key1 = await store.createKey();
    const key2 = await store.createKey();
    expect(await store.getKeyList()).toContain(key1);
    expect(await store.getKeyList()).toContain(key2);

    await store.removeKey({ key: key1 });

    expect(await store.getKeyList()).not.toContain(key1);
});

test("exportRawKey", async () => {
    const store = await LocalKeyStore.createForTest();
    const key = await store.createKey({ passphrase: "satoshi" });
    const privateKey = await store.exportRawKey({
        key,
        passphrase: "satoshi"
    });

    const publicKey = getPublicFromPrivate(privateKey);
    const accountId = getAccountIdFromPublic(publicKey);
    expect(accountId).toBe(key);
});

test("sign", async () => {
    const store = await LocalKeyStore.createForTest();
    const key = await store.createKey();
    const publicKey = await store.getPublicKey({ key });
    if (publicKey == null) {
        throw Error("Cannot get the public key");
    }
    const message =
        "00000000c0dec6a100000000c0dec6a100000000c0dec6a100000000c0dec6a1";
    const signature = await store.sign({
        key,
        message
    });
    expect(verifyEd25519(message, signature, publicKey)).toBe(true);
});
