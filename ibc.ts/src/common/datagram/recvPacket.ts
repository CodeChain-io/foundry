import { Packet } from ".";

const RLP = require("rlp");

export class RecvPacketDatagram {
    private packet: Packet;
    private proof: Buffer;
    private proofHeight: number;
    private ack: Buffer;

    public constructor({
        packet,
        proof,
        proofHeight,
        ack
    }: {
        packet: Packet;
        proof: Buffer;
        proofHeight: number;
        ack: Buffer;
    }) {
        this.packet = packet;
        this.proof = proof;
        this.proofHeight = proofHeight;
        this.ack = ack;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [
            14,
            this.packet.toEncodeObject(),
            this.proof,
            this.proofHeight,
            this.ack
        ];
    }
}
