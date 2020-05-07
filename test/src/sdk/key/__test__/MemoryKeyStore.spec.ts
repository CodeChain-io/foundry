import { expect } from "chai";
import "mocha";
import { getPublicFromPrivate, verifyEd25519 } from "../../utils";
import { MemoryKeyStore } from "../MemoryKeyStore";

it("createKey", async () => {
    const store = new MemoryKeyStore();
    // await expect(store.createKey()).resolves.toEqual(expect.anything());
});

it("removeKey", async () => {
    const store = new MemoryKeyStore();
    const key1 = await store.createKey();
    expect(await store.removeKey({ key: key1 })).true;
    expect(await store.removeKey({ key: key1 })).false;
});

it("getKeyList", async () => {
    const store = new MemoryKeyStore();
    const key1 = await store.createKey();
    const key2 = await store.createKey();
    expect(await store.getKeyList()).contains(key1);
    expect(await store.getKeyList()).contains(key2);

    await store.removeKey({ key: key1 });

    expect(await store.getKeyList()).not.contains(key1);
});

it("exportRawKey", async () => {
    const store = new MemoryKeyStore();
    const key = await store.createKey({ passphrase: "satoshi" });
    const privateKey = await store.exportRawKey({
        key,
        passphrase: "satoshi"
    });

    const publicKey = getPublicFromPrivate(privateKey);
    const storedPubKey = await store.getPublicKey({ key });
    expect(publicKey).equal(storedPubKey);
});

it("sign", async () => {
    const store = new MemoryKeyStore();
    const key = await store.createKey();
    const publicKey = (await store.getPublicKey({ key }))!;
    const message =
        "00000000c0dec6a100000000c0dec6a100000000c0dec6a100000000c0dec6a1";
    const signature = await store.sign({
        key,
        message
    });
    expect(verifyEd25519(message, signature, publicKey)).true;
});
