const RLP = require("rlp");

export class ConnOpenAckDatagram {
    private identifier: string;
    private proofTry: Buffer;
    private proofConsensus: Buffer;
    private proofHeight: number;
    private consensusHeight: number;

    public constructor({
        identifier,
        proofTry,
        proofConsensus,
        proofHeight,
        consensusHeight
    }: {
        identifier: string;
        proofTry: Buffer;
        proofConsensus: Buffer;
        proofHeight: number;
        consensusHeight: number;
    }) {
        this.identifier = identifier;
        this.proofTry = proofTry;
        this.proofConsensus = proofConsensus;
        this.proofHeight = proofHeight;
        this.consensusHeight = consensusHeight;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [
            5,
            this.identifier,
            this.proofTry,
            this.proofConsensus,
            this.proofHeight,
            this.consensusHeight
        ];
    }
}
