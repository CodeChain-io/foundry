import { SDK, AssetMintTransaction, H256, Parcel, H160, U256, PaymentTransaction, H512, SetRegularKeyTransaction } from "../";
import { privateKeyToAddress } from "../src/utils";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
const address = new H160(privateKeyToAddress(secret.value));

export const mintAsset = async ({ metadata, amount, lockScriptHash, parameters, registrar }) => {
    const assetMintTransaction = new AssetMintTransaction({
        nonce: 1,
        metadata,
        lockScriptHash,
        parameters,
        amount,
        registrar,
    });
    const nonce = await sdk.getNonce(address);
    const fee = new U256(10);
    const networkId = 17;

    const parcel = new Parcel(nonce, fee, networkId, assetMintTransaction);
    const parcelHash = await sdk.sendSignedParcel(parcel.sign(secret));

    return {
        parcelHash,
        assetMintTransaction
    };
};

export const payment = async () => {
    const nonce = await sdk.getNonce(address);
    const t = new PaymentTransaction({
        nonce: nonce.increase(),
        sender: address,
        receiver: address,
        value: new U256(0)
    });
    const fee = new U256(10);
    const networkId = 17;
    const p = new Parcel(nonce, fee, networkId, t).sign(secret);
    return await sdk.sendSignedParcel(p);
};

export const paymentTwice = async () => {
    const fee = new U256(10);
    const networkId = 17;
    const value = new U256(0);

    const nonce1 = await sdk.getNonce(address);
    const nonce2 = nonce1.increase();
    const nonce3 = nonce2.increase();
    const nonce4 = nonce3.increase();

    const t1 = new PaymentTransaction({
        nonce: nonce2,
        sender: address,
        receiver: address,
        value});
    const p1 = new Parcel(nonce1, fee, networkId, t1).sign(secret);
    await sdk.sendSignedParcel(p1);

    const t2 = new PaymentTransaction({
        nonce: nonce4,
        sender: address,
        receiver: address,
        value
    });
    const p2 = new Parcel(nonce3, fee, networkId, t2).sign(secret);
    await sdk.sendSignedParcel(p2);
};

export const setRegularKey = async () => {
    const nonce = await sdk.getNonce(address);
    const key = new H512("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    const t = new SetRegularKeyTransaction({
        address,
        nonce: nonce.increase(),
        key
    });
    const fee = new U256(10);
    const networkId = 17;
    const p = new Parcel(nonce, fee, networkId, t);
    return await sdk.sendSignedParcel(p.sign(secret));
};
