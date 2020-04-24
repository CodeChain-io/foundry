// Copyright 2019-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

import { SDK } from "../src/sdk";
import { Signer } from "../src/helper/spawn";

const privateKeys = [
  "df7fc69a4dee82b9efa6d48d45dc1b541c54eb61e3af5acf577f07cdb706aa1d9871a148fbc474f82ad1c93140b9a24b7c2e8944158816bab6f828c5f4502461",
  // tccqxv8rg2gl0z8f7p268ynzs9e5f9hct5fgs2cs946kmuz33052qjxz423n4y
  "be173d1f8b1f76b8ae0d15bd6c4beeb664ef12655310dd04c36945dc86b075a1dc7d598adaea461bf3c85ae727ae9d653d315f07099a200d5fbef49b3ffd5d25",
  // tccq8w86kv2mt4yvxlnepdwwfawn4jn6v2lquye5gqdt7l0fxell4wj2pwx85u
  "e24f42d28d82a13b2031397df561cf1066b3eeaa959fa2a4de763e657efb3926bc7729f91c57f575c9555db790a979ef8143ea29488f3b5e498e3c2c00b1d391",
  // tccqx78w20er3tl2awf24wm0y9f08hczsl299yg7w67fx8rctqqk8fezq580z2
  "2c93f98e02993a3a527685e7d84fef83ccf1d9100cc9d4ac43f82d833bc07a0ee370aadce8856a5f7424840183631b99a24caab7d0a50f7ca8e0a3c91f81b167",
  // tccq83hp2kuazzk5hm5yjzqrqmrrwv6yn92klg22rmu4rs28jglsxckwcr2zhe
  "0cb0cc8f61268961ba7a2c702db66c9a8970fe5ce0b3d87a564895519df33c4611cb585bcd00ae47e410474987d4d8129de1305d48d5886be62b4ff50f284af3",
  // tccqygukkzme5q2u3lyzpr5np75mqffmcfst4ydtzrtuc45lag09p90xvdtzhg
  "16d7a9c195796975aa384a156a14150a9d3752d2a9cd290610d12a215d928f1c6ce16180aa209d0349f8eb6702cf5d061e8185ef6ea4728d86ee2b20a850640d",
  // tccq9kwzcvq4gsf6q6flr4kwqk0t5rpaqv9aah2gu5dsmhzkg9g2pjq6yypwce
  "da4a1559b7ab680d1ae5bc322b0c1ce56bff4c0d811a3f03b24c8b41a71a692bc0943a2a2635835e9eac7a095ec65805190c0dfeaddaad20f01cbf0f290457d2",
  // tccq8qfgw32yc6cxh5743aqjhkxtqz3jrqdl6ka4tfq7qwt7refq3tayv8tzs0
  "ad36796cdf5d9dc9ddfa0baed6c427719f3cb9c86a1e14befe9ca4878e175d17fba867347b420780ebc65db9659122d22d290dff9d69e64cabe62cde482a28e5",
  // tccq8a6see50dpq0q8tcewmjev3ytfz62gdl7wknejv40nzehjg9g5w2wvscnd
  "ff3541f2714fc90c10712eb76f1ae06645570d18aae4ed8427b7d5eab1b1cddc1a6b1c4b3849818d860c79bb07a748268fe86baec88ac071d9dc592c21ba5abf",
  // tccqydxk8zt8pycrrvxp3umkpa8fqngl6rt4myg4sr3m8w9jtpphfdt7usuu2a
  "579d2f2d337b0d9421db3e8dc8505c9de8408968036727eb29e873389a153aecb3d43f08b7e2131dc7ad1b929d74c3b2d78585e6f65ea286d8c7b8e2c1123b16"
  // tccqxeag0cgkl3px8w845de98t5cwed0pv9umm9ag5xmrrm3ckpzga3vf89tuc
];

export const validators: Signer[] = privateKeys.map(privateKey => {
  const publicKey = SDK.util.getPublicFromPrivate(privateKey);
  const address = SDK.Core.classes.Address.fromPublic(
    publicKey,
    {
      networkId: "tc"
    }
  );
  return { privateKey, publicKey, address };
});
