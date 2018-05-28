import { AssetMintTransaction, H256 } from "../src";

const t = new AssetMintTransaction({
    metadata: "",
    lockScriptHash: new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"),
    parameters: [],
    amount: 100,
    registrar: null
    nonce: 0,
});

console.log("hash", t.hash());
console.log("asset scheme receiver", t.getAssetSchemeAddress());
