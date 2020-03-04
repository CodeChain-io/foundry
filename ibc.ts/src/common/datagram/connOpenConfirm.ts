const RLP = require("rlp");

export class ConnOpenConfirmDatagram {
    private identifier: string;
    private proofAck: Buffer;
    private proofHeight: number;

    public constructor({
        identifier,
        proofAck,
        proofHeight
    }: {
        identifier: string;
        proofAck: Buffer;
        proofHeight: number;
    }) {
        this.identifier = identifier;
        this.proofAck = proofAck;
        this.proofHeight = proofHeight;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [6, this.identifier, this.proofAck, this.proofHeight];
    }
}
