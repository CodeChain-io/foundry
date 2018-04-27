import U256 from "./U256";
import Transaction from "./Transaction";

class SignedTransaction {
    private unsigned: Transaction;
    private v: number;
    private r: U256;
    private s: U256;

    constructor(unsigned: Transaction, v: number, r: U256, s: U256) {
        this.unsigned = unsigned;
        this.v = v;
        this.r = r;
        this.s = s;
    }

    signature() {
        const { v, r, s } = this;
        return { v, r, s };
    }
}

export default SignedTransaction;
