const RLP = require("rlp");

export class ChanOpenConfirmDatagram {
    private channelIdentifier: string;
    private proofAck: Buffer;
    private proofHeight: number;

    public constructor({
        channelIdentifier,
        proofAck,
        proofHeight
    }: {
        channelIdentifier: string;
        proofAck: Buffer;
        proofHeight: number;
    }) {
        this.channelIdentifier = channelIdentifier;
        this.proofAck = proofAck;
        this.proofHeight = proofHeight;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [10, this.channelIdentifier, this.proofAck, this.proofHeight];
    }
}
