import { H256, H512, U64 } from "codechain-sdk/lib/core/classes";

export enum ConsensusStep {
    Propose = 0x00,
    Prevote = 0x01,
    Precommit = 0x02,
    Commit = 0x03
}

export function isStep(val: number): val is ConsensusStep {
    return (
        val === ConsensusStep.Propose ||
        val === ConsensusStep.Prevote ||
        val === ConsensusStep.Precommit ||
        val === ConsensusStep.Commit
    );
}

export interface ConsensusMessage {
    on: {
        step: {
            height: U64;
            view: U64;
            step: ConsensusStep;
        };
        blockHash: H256 | null;
    };
    signature: H512;
    signerIndex: U64;
}
