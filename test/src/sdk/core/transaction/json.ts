import {
    Address,
    H160,
    H256,
    H512,
    SignedTransaction,
    Transaction,
    U64
} from "../classes";
import { SignedTransactionJSON } from "../SignedTransaction";
import { Custom } from "./Custom";
import { Pay } from "./Pay";

export function fromJSONToTransaction(result: any): Transaction {
    const { seq, fee, networkId, action } = result;
    let tx;
    switch (action.type) {
        case "pay": {
            const receiver = Address.ensure(action.receiver);
            const quantity = new U64(action.quantity);
            tx = new Pay(receiver, quantity, networkId);
            break;
        }
        case "custom": {
            const handlerId = U64.ensure(action.handlerId);
            const bytes = Buffer.from(action.bytes);
            tx = new Custom(
                {
                    handlerId,
                    bytes
                },
                networkId
            );
            break;
        }
        default:
            throw Error(`Unexpected action: ${action}`);
    }
    if (seq != null) {
        tx.setSeq(seq);
    }
    if (fee != null) {
        tx.setFee(fee);
    }
    return tx;
}

/**
 * Create a SignedTransaction from a SignedTransaction JSON object.
 * @param data A SignedTransaction JSON object.
 * @returns A SignedTransaction.
 */
export function fromJSONToSignedTransaction(data: SignedTransactionJSON) {
    const {
        sig,
        signerPublic,
        blockNumber,
        blockHash,
        transactionIndex
    } = data;
    if (typeof sig !== "string") {
        throw Error("Unexpected type of sig");
    }
    if (blockNumber != null && blockHash != null && transactionIndex != null) {
        return new SignedTransaction(
            fromJSONToTransaction(data),
            sig,
            signerPublic,
            blockNumber,
            new H256(blockHash),
            transactionIndex
        );
    } else {
        return new SignedTransaction(
            fromJSONToTransaction(data),
            sig,
            signerPublic
        );
    }
}
