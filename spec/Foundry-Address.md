Foundry embeds the ed25519 signature scheme.
This scheme is used for the block signer and used by the modules.
It means when Foundry requires your account, it needs the public key of your secret.

The public key of the ed25519 scheme is 256 bits binary.
Since the binary itself is not human readable, the key should be encoded to a plain text.

The hex encoding is possible to represent the key, but the property of the hex encoding doesn't meet the requirements of Foundry.

The properties of Foundry requires are
* easy to pronounce for easy identification.
* highly efficient.
* easily detect typoes.

So Foundry uses a complex encoding.
The encoded text of the public key is called as the foundry address.
The details of encoding are depends on the version.
But in all versions, the last character is the hex encoded version.

## Version 0

Version 0 consists of four parts.
* name of the address
* public key
* network id
* version
And each part uses different encoding schemes according to their purpose.

### Name of the address
The name of the address prevents ambiguity, both for humans and computers.
It's a checksum computed from all other information.
The checksum uses [the Geohash represntation](https://en.wikipedia.org/wiki/Geohash#Textual_representation) which removes the ambiguity.
That representation uses all lower case except "a", "i", "l", and "o" so it's easy to read for human.

The most encoding scheme presents the checksum at last.
However, this scheme presents the checksum at the beginning because it's also a human-readable part of the address.

The below description is the way to calculate the checksum.
```
pubkey := p_0 .... p_31
network_id := n_0 . n_1
c_0 := p_0 . p_1 . p_2 . p_3 . p_4
c_1 := p_5 . p_6 . p_7 . p_8 . p_9
c_2 := p_10 . p_11 . p_12 . p_13 . p_14
c_3 := p_15 . p_16 . p_17 . p_18 . p_19
c_4 := p_20 . p_21 . p_22 . p_23 . p_24
c_5 := p_25 . p_26 . p_27 . p_28 . p_29
c_6 := n_0 . n_1 . version . p_30 . p_31
checksum := c_0 ^ c_1 ^ c_2 ^ c_3 ^ c_4 ^ c_5 ^ c_6
```
This result is the five characters, but the checksum is reresented as eight characters because the Geohash representation is 5-8 encoding scheme.

### public key
The public key is encoded with [base64url](https://tools.ietf.org/html/rfc4648#section-5) without padding.
Base64 is a highly efficient scheme, but, unlike base64 this scheme is safe to use as a URL or filename.
And we don't need padding because we know the length of the public key.
Since the public key is 256 bits, the public key part is 43 characters.

### network id
The network id is two byte characters that represents the network id.

### version
The version is hex encoded one ascii character.
It is awalys "0"(0x48).

### examples of version 0
If the public key is `d7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c`, network id is `tc`,
the name of the address is `01sv1ngs`, the public key is represented as `16bSZoN8HFkTg7kNg1Boue1Y3TvOvW4oWRH1jkDOQTw`.
So the address is `01sv1ngs16bSZoN8HFkTg7kNg1Boue1Y3TvOvW4oWRH1jkDOQTwtc0`.
