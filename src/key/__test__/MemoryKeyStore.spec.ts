import {
    recoverEcdsa,
    verifyEcdsa,
    getPublicFromPrivate,
    getAccountIdFromPublic
} from "../../utils";
import { MemoryKeyStore } from "../MemoryKeyStore";

test("createKey", async () => {
    const store = new MemoryKeyStore();
    await expect(store.asset.createKey()).resolves.toEqual(expect.anything());
});

test("removeKey", async () => {
    const store = new MemoryKeyStore();
    const key1 = await store.asset.createKey();
    expect(await store.asset.removeKey({ key: key1 })).toBe(true);
    expect(await store.asset.removeKey({ key: key1 })).toBe(false);
});

test("getKeyList", async () => {
    const store = new MemoryKeyStore();
    const key1 = await store.asset.createKey();
    const key2 = await store.asset.createKey();
    expect(await store.asset.getKeyList()).toContain(key1);
    expect(await store.asset.getKeyList()).toContain(key2);

    await store.asset.removeKey({ key: key1 });

    expect(await store.asset.getKeyList()).not.toContain(key1);
});

test("exportRawKey", async () => {
    const store = new MemoryKeyStore();
    const key = await store.platform.createKey({ passphrase: "satoshi" });
    const privateKey = await store.platform.exportRawKey({
        key,
        passphrase: "satoshi"
    });

    const publicKey = getPublicFromPrivate(privateKey);
    const accountId = getAccountIdFromPublic(publicKey);
    expect(accountId).toBe(key);
});

test("sign", async () => {
    const store = new MemoryKeyStore();
    const key = await store.asset.createKey();
    const publicKey = await store.asset.getPublicKey({ key });
    const signature = await store.asset.sign({
        key,
        message: "hello"
    });
    const r = `${signature.substr(0, 64)}`;
    const s = `${signature.substr(64, 64)}`;
    const v = Number.parseInt(signature.substr(128, 2), 16);

    expect(verifyEcdsa("hello", { r, s, v }, publicKey)).toBe(true);
    expect(recoverEcdsa("hello", { r, s, v })).toEqual(publicKey);
});
