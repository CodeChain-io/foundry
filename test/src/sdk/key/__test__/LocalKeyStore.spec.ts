import { expect } from "chai";
import "mocha";
import { getPublicFromPrivate, verifyEd25519 } from "../../utils";
import { LocalKeyStore } from "../LocalKeyStore";

it("createKey", async () => {
    const store = await LocalKeyStore.createForTest();
    // await expect(store.createKey()).resolves.toEqual(expect.anything());
});

it("removeKey", async () => {
    const store = await LocalKeyStore.createForTest();
    const key1 = await store.createKey();
    expect(await store.removeKey({ key: key1 })).true;
    expect(await store.removeKey({ key: key1 })).false;
});

it("getKeyList", async () => {
    const store = await LocalKeyStore.createForTest();
    const key1 = await store.createKey();
    const key2 = await store.createKey();
    expect(await store.getKeyList()).contains(key1);
    expect(await store.getKeyList()).contains(key2);

    await store.removeKey({ key: key1 });

    expect(await store.getKeyList()).not.contains(key1);
});

it("exportRawKey", async () => {
    const passphrase = "satoshi";
    const store = await LocalKeyStore.createForTest();
    const key = await store.createKey({ passphrase });
    const privateKey = await store.exportRawKey({
        key,
        passphrase
    });

    const publicKey = getPublicFromPrivate(privateKey);
    const storedKey = await store.getPublicKey({ key, passphrase });
    expect(publicKey).equal(storedKey);
});

it("sign", async () => {
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
    expect(verifyEd25519(message, signature, publicKey)).true;
});
