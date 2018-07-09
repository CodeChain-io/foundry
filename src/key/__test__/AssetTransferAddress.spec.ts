import { H256 } from "../../core/H256";

import { AssetTransferAddress } from "../AssetTransferAddress";

test("AssetTransferAddress.fromLockScriptHash - mainnet", () => {
    const lockScriptHash = new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3");
    const address = AssetTransferAddress.fromLockScriptHash(lockScriptHash);
    expect(address.value).toMatch(/^cca[a-z0-9]+$/);
});

test("AssetTransferAddress.fromLockScriptHash - testnet", () => {
    const lockScriptHash = new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3");
    const address = AssetTransferAddress.fromLockScriptHash(lockScriptHash, { isTestnet: true });
    expect(address.value).toMatch(/^tca[a-z0-9]+$/);
});

test("AssetTransferAddress.fromLockScriptHash - invalid version", () => {
    const lockScriptHash = new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3");
    expect(() => {
        AssetTransferAddress.fromLockScriptHash(lockScriptHash, { version: 1 });
    }).toThrow("Unsupported version for asset transfer address: 1");
});

test("AssetTransferAddress.fromPublicKeyHash - mainnet", () => {
    const publicKeyHash = new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3");
    const address = AssetTransferAddress.fromPublicKeyHash(publicKeyHash);
    expect(address.value).toMatch(/^cca[a-z0-9]+$/);
});

test("AssetTransferAddress.fromPublicKeyHash - testnet", () => {
    const publicKeyHash = new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3");
    const address = AssetTransferAddress.fromPublicKeyHash(publicKeyHash, { isTestnet: true });
    expect(address.value).toMatch(/^tca[a-z0-9]+$/);
});

test("AssetTransferAddress.fromAddress - mainnet", () => {
    const address = AssetTransferAddress.fromAddress("ccaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvcev083x");
    expect(address.payload).toEqual(new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"));
});

test("AssetTransferAddress.fromAddress - testnet", () => {
    const address = AssetTransferAddress.fromAddress("tcaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvctt5eze");
    expect(address.payload).toEqual(new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"));
});

test("AssetTransferAddress.fromAddress - invalid checksum", () => {
    expect(() => {
        AssetTransferAddress.fromAddress("ccaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvcqqqqqq");
    }).toThrow();
});

