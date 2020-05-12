import {
    blake128,
    blake128WithKey,
    blake160,
    blake160WithKey,
    blake256,
    blake256WithKey,
    ripemd160
} from "../src";
import "mocha";
import { expect } from "chai";

it("blake128", () => {
    const hash = blake128("deadbeef");

    expect(/^[0-9a-fA-F]{32}$/.test(hash)).true;
    expect(hash).equal("6f5ca1fbef92681581176e231a9ff125");
});

it("blake128WithKey", () => {
    const hash = blake128WithKey("deadbeef", new Uint8Array(16));

    expect(/^[0-9a-fA-F]{32}$/.test(hash)).true;
    expect(hash).equal("b98324686a2c8327451b02f3a280c0f2");
});

it("blake160", () => {
    const hash = blake160("deadbeef");

    expect(/^[0-9a-fA-F]{40}$/.test(hash)).true;
    expect(hash).equal("e8c8d008ee369e385cff36246425c7b30696a2b1");
});

it("blake160WithKey", () => {
    const hash = blake160WithKey("deadbeef", new Uint8Array(16));

    expect(/^[0-9a-fA-F]{40}$/.test(hash)).true;
    expect(hash).equal("850b2b598a7782fe904860fbec66d396697fa47b");
});

it("blake256", () => {
    const hash = blake256("deadbeef");

    expect(/^[0-9a-fA-F]{64}$/.test(hash)).true;
    expect(hash).equal(
        "f3e925002fed7cc0ded46842569eb5c90c910c091d8d04a1bdf96e0db719fd91"
    );
});

it("blake256WithKey", () => {
    const hash = blake256WithKey("deadbeef", new Uint8Array(16));

    expect(/^[0-9a-fA-F]{64}$/.test(hash)).true;
    expect(hash).equal(
        "f247b4a8963b51a380cd5065a62c5b847fc84de899c41cd9d9dd0133d8980602"
    );
});

it("ripemd160", () => {
    const hash = ripemd160("deadbeef");

    expect(/^[0-9a-fA-F]{40}$/.test(hash)).true;
    expect(hash).equal("226821c2f5423e11fe9af68bd285c249db2e4b5a");
});
