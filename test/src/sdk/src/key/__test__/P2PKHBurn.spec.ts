import { H160 } from "codechain-primitives";

import { blake160 } from "../../utils";

import { P2PKHBurn } from "../P2PKHBurn";

test("getLockScriptHash()", () => {
    const hash = new H160(blake160(P2PKHBurn.getLockScript()));
    expect(P2PKHBurn.getLockScriptHash()).toEqual(hash);
});
