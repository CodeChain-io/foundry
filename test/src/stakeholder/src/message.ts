import { H256, H512, U64 } from "codechain-sdk/lib/core/classes";

import { decodeH256, decodeH512, decodeU64, decodeUInt } from "./util";

export enum ConsensusStep {
    Propose = 0x00,
    Prevote = 0x01,
    Precommit = 0x02,
    Commit = 0x03
}

function isStep(val: number): val is ConsensusStep {
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

export function messageToEncodeObject(message: ConsensusMessage) {
    return [
        [
            [
                message.on.step.height.toEncodeObject(),
                message.on.step.view.toEncodeObject(),
                message.on.step.step
            ],
            message.on.blockHash == null
                ? []
                : [message.on.blockHash.toEncodeObject()]
        ],
        message.signature.toEncodeObject(),
        message.signerIndex.toEncodeObject()
    ];
}

export function decodeMessage(list: any[]): ConsensusMessage {
    if (list.length !== 3) {
        throw new Error(
            "The raw value of ConsensusMessage should be a list of length 3"
        );
    }
    if (!Array.isArray(list[0]) || list[0].length !== 2) {
        throw new Error("The raw value of VoteOn should be a list of length 3");
    }
    if (!Array.isArray(list[0][0]) || list[0][0].length !== 3) {
        throw new Error(
            "The raw value of VoteStep should be a list of length 3"
        );
    }
    const step: number = decodeUInt(list[0][0][2]);
    if (!isStep(step)) {
        throw new Error("The consensus step should be in valid range");
    }

    const voteStep: ConsensusMessage["on"]["step"] = {
        height: decodeU64(list[0][0][0]),
        view: decodeU64(list[0][0][1]),
        step
    };

    if (!Array.isArray(list[0][1])) {
        throw new Error("The raw value of blockHash should be a list");
    }
    let blockHash: H256 | null;
    if (list[0][1].length === 0) {
        blockHash = null;
    } else if (list[0][1].length === 1) {
        blockHash = decodeH256(list[0][1][0]);
    } else {
        throw new Error(
            "The raw value of blockHash should be a list of length 0 or 1"
        );
    }

    const signature = decodeH512(list[1]);
    const signerIndex = decodeU64(list[2]);

    return {
        on: {
            step: voteStep,
            blockHash
        },
        signature,
        signerIndex
    };
}
