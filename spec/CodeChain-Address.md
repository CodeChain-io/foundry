When transferring CCC, the sender must know the recipient's lock script hash and parameters. An address is a converted form of the lock script hash and parameters, and it has some benefits.

 * An address includes a checksum. There is a high probability that a mistyped address is invalid.
 * An address is case-insensitive alphanumeric, which is easy to speak aloud or type on the mobile phone. It also makes it efficient to generate QR codes.

## Bech32

CodeChain adopted [Bitcoin's Bech32 Specification](https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki#bech32). The differences from Bitcoin are the following:

 * CodeChain has no separator.

Address formats are not a core part.

## 1. Platform Account Address Format

HRP: `"ccc"` for Mainnet, `"tcc"` for Testnet.

Data Part: `version` . `body`

### Version 0 (0x00)

No longer available. Any version 0 address will be rejected in the latest clients.

### Version 1 (0x01)

Data body: `Account ID` (20 bytes)

Account ID is the result of blake160 over a public key(64 bytes uncompressed form).

---

## Address examples

* Address: `cccqx37a03l3axrz3qmtdywgjuyuvr099dueuqvjxp3`
  * version = `1`
  * payload(Account ID) = `a3eebe3f8f4c31441b5b48e44b84e306f295bccf`
