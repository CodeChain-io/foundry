# Curve

Foundry uses [Curve25519/Edward25519](https://cr.yp.to/ecdh/curve25519-20060209.pdf) because it provides the efficient algorithms for key exchange and signing.

# Signature Algorithm

Foundry uses [Ed25519 signature](https://ed25519.cr.yp.to/ed25519-20110926.pdf) as its digital signature algorithm instead of more conventional ECDSA. Ed25519 scheme is a EdDSA digital signature scheme applied to the Edward25519 elliptic curve.

Ed25519 signature has a couple of [nice properties](https://ed25519.cr.yp.to/)