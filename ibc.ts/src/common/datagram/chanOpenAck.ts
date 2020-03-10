const RLP = require("rlp");

export class ChanOpenAckDatagram {
    private channelIdentifier: string;
    private counterpartyVersion: string;
    private proofTry: Buffer;
    private proofHeight: number;

    public constructor({
        channelIdentifier,
        counterpartyVersion,
        proofTry,
        proofHeight
    }: {
        channelIdentifier: string;
        counterpartyVersion: string;
        proofTry: Buffer;
        proofHeight: number;
    }) {
        this.channelIdentifier = channelIdentifier;
        this.counterpartyVersion = counterpartyVersion;
        this.proofTry = proofTry;
        this.proofHeight = proofHeight;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [
            9,
            this.channelIdentifier,
            this.counterpartyVersion,
            this.proofTry,
            this.proofHeight
        ];
    }
}
