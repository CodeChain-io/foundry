// Copyright 2018-2020 Kodebox, Inc.
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

import { SDK } from "../sdk";

export const faucetSecret =
    "9af28f6fd6a1170dbee2cb8c34abab0408e6d811d212cdcde23f72473eb0d97ad7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c";
export const faucetPublic = SDK.util.getPublicFromPrivate(faucetSecret); // d7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c
export const faucetAddress = SDK.Core.classes.Address.fromPublic(faucetPublic, {
    networkId: "tc"
}); // 01sv1ngs16bSZoN8HFkTg7kNg1Boue1Y3TvOvW4oWRH1jkDOQTwtc0

export const aliceSecret =
    "65cc9daeecc3bf26befc6b4c8fba3f10d910dbe2e086669fa62eb812cb4254175f6fcadd723f3c94970101d91b4043f953b8f939e14b75843b2f189bb5264f55";
export const alicePublic = SDK.util.getPublicFromPrivate(aliceSecret); // 5f6fcadd723f3c94970101d91b4043f953b8f939e14b75843b2f189bb5264f55
export const aliceAddress = SDK.Core.classes.Address.fromPublic(alicePublic, {
    networkId: "tc"
}); // 4cnj73b1X2_K3XI_PJSXAQHZG0BD-VO4-TnhS3WEOy8Ym7UmT1Utc0

export const bobSecret =
    "26aa96a941e9764513155c80170b876ff62cd4ea790bbada5839c39f1e2542b53ab23fad7c9a4f1f54dbf44ff9294d2b5ab42d15f161e406048d5abf7e5dad94";
export const bobPublic = SDK.util.getPublicFromPrivate(bobSecret); // 3ab23fad7c9a4f1f54dbf44ff9294d2b5ab42d15f161e406048d5abf7e5dad94
export const bobAddress = SDK.Core.classes.Address.fromPublic(bobPublic, {
    networkId: "tc"
}); // fys3db1kOrI_rXyaTx9U2_RP-SlNK1q0LRXxYeQGBI1av35drZQtc0

export const carolSecret =
    "f65ea73ec07bd1ca9a1c12945bdbc885f5cf3143227804c6b0a591f4ea5887b2e83c0184ed9acc66868a7be2fbe901eecfe7c054450bbb8d24328e0116ea5e0c";
export const carolPublic = SDK.util.getPublicFromPrivate(carolSecret); // e83c0184ed9acc66868a7be2fbe901eecfe7c054450bbb8d24328e0116ea5e0c
export const carolAddress = SDK.Core.classes.Address.fromPublic(carolPublic, {
    networkId: "tc"
}); // whxw3vrh6DwBhO2azGaGinvi--kB7s_nwFRFC7uNJDKOARbqXgwtc0

export const daveSecret =
    "d1798178ca055593a4618f2ed313f5568221a9e574e850a8a464025a3fb720aa200c2fe942fdbe9143323ed264d0e39e7b321ca33c78bfa78a92576e00dc9ebd";
export const davePublic = SDK.util.getPublicFromPrivate(daveSecret); // 200c2fe942fdbe9143323ed264d0e39e7b321ca33c78bfa78a92576e00dc9ebd
export const daveAddress = SDK.Core.classes.Address.fromPublic(davePublic, {
    networkId: "tc"
}); // nxcmkryvIAwv6UL9vpFDMj7SZNDjnnsyHKM8eL-nipJXbgDcnr0tc0

export const invalidSecret =
    "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
export const invalidAddress =
    "fjjh0000AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAtc0";

export const validator0Secret =
    "4ca2cbc987cd76b393f11124fe7145fdc680e311a1ed9dee060e7c3fbeb8943e0a6902c51384a15d1062cac3a4e62c8d0c2eb02b4de7fa0a304ce4f88ea482d0";
export const validator0Public = SDK.util.getPublicFromPrivate(validator0Secret);
// 0a6902c51384a15d1062cac3a4e62c8d0c2eb02b4de7fa0a304ce4f88ea482d0
export const validator0Address = SDK.Core.classes.Address.fromPublic(
    validator0Public,
    { networkId: "tc" }
); // rjmxg19kCmkCxROEoV0QYsrDpOYsjQwusCtN5_oKMEzk-I6kgtAtc0

export const validator1Secret =
    "ff85044d118b635f23e93c5280648e6b3607781aa770aea4d44a9a3d5703867d0473f782c3aec053c37fe2bccefa9298dcf8ae3dc2262ae540a14a580ff773e6";
export const validator1Public = SDK.util.getPublicFromPrivate(validator1Secret);
// 0473f782c3aec053c37fe2bccefa9298dcf8ae3dc2262ae540a14a580ff773e6
export const validator1Address = SDK.Core.classes.Address.fromPublic(
    validator1Public,
    { networkId: "tc" }
); // szff1322BHP3gsOuwFPDf-K8zvqSmNz4rj3CJirlQKFKWA_3c-Ytc0

export const validator2Secret =
    "0fecf401905fe5a6bd9d9e67bbccaff7711d2a060b3b0019550285d62f4995d02502d5e6210679a19e45f3c0f93257e7a327baaf5f403f5ca1ab2685a9e1724e";
export const validator2Public = SDK.util.getPublicFromPrivate(validator2Secret);
// 2502d5e6210679a19e45f3c0f93257e7a327baaf5f403f5ca1ab2685a9e1724e
export const validator2Address = SDK.Core.classes.Address.fromPublic(
    validator2Public,
    { networkId: "tc" }
); // qwfj0xwkJQLV5iEGeaGeRfPA-TJX56Mnuq9fQD9coasmhanhck4tc0

export const validator3Secret =
    "16f9ae3249d1499f6a5da3493574f27dab800fd0998634be8c010e21505f97aee909f311fd115ee412edcfcde88cc507370101f7635a67b9cb45390f1ccb4b5e";
export const validator3Public = SDK.util.getPublicFromPrivate(validator3Secret);
// e909f311fd115ee412edcfcde88cc507370101f7635a67b9cb45390f1ccb4b5e
export const validator3Address = SDK.Core.classes.Address.fromPublic(
    validator3Public,
    { networkId: "tc" }
); // dbqtds3w6QnzEf0RXuQS7c_N6IzFBzcBAfdjWme5y0U5DxzLS14tc0

export const stakeActionHandlerId = 2;
