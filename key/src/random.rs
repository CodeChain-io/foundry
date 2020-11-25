// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{Ed25519KeyPair, Generator, KeyPairTrait, SealKeyPair, X25519KeyPair, X25519Private, X25519Public};
use never_type::Never;
use rand::rngs::OsRng;
#[cfg(test)]
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use sodiumoxide::crypto::box_::gen_keypair as gen_curve25519xsalsa20poly1305;
use sodiumoxide::crypto::kx::gen_keypair as gen_x25519;
use sodiumoxide::crypto::sign::gen_keypair as gen_ed25519;
#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
use std::{mem, thread};

pub struct Random;

#[cfg(test)]
thread_local! {
    static RNG: RefCell<XorShiftRng> = {
        let thread_id: [u8; 8] = unsafe { mem::transmute(thread::current().id()) };
        let mut seed: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7];
        seed[0..8].copy_from_slice(&thread_id);
        RefCell::new(XorShiftRng::from_seed(seed))
    };
}

impl Generator<Ed25519KeyPair> for Random {
    type Error = ::std::io::Error;

    //FIXME: there is no distinction between the two generate functions
    #[cfg(not(test))]
    fn generate(&mut self) -> Result<Ed25519KeyPair, Self::Error> {
        let mut rng = OsRng::new()?;
        match rng.generate() {
            Ok(pair) => Ok(pair),
            Err(never) => match never {}, // LLVM unreachable
        }
    }

    #[cfg(test)]
    fn generate(&mut self) -> Result<Ed25519KeyPair, Self::Error> {
        RNG.with(|rng| {
            match rng.borrow_mut().generate() {
                Ok(pair) => Ok(pair),
                Err(never) => match never {}, // LLVM unreachable
            }
        })
    }
}

impl Generator<Ed25519KeyPair> for OsRng {
    type Error = Never;

    fn generate(&mut self) -> Result<Ed25519KeyPair, Self::Error> {
        let (publ, sec) = gen_ed25519();
        Ok(Ed25519KeyPair::from_keypair(sec.into(), publ.into()))
    }
}

impl Generator<Ed25519KeyPair> for XorShiftRng {
    type Error = Never;

    fn generate(&mut self) -> Result<Ed25519KeyPair, Self::Error> {
        let (publ, sec) = gen_ed25519();
        Ok(Ed25519KeyPair::from_keypair(sec.into(), publ.into()))
    }
}

impl Generator<X25519KeyPair> for Random {
    type Error = ::std::io::Error;

    //FIXME: there is no distinction between the two generate functions
    #[cfg(not(test))]
    fn generate(&mut self) -> Result<X25519KeyPair, Self::Error> {
        let mut rng = OsRng::new()?;
        match rng.generate() {
            Ok(pair) => Ok(pair),
            Err(never) => match never {}, // LLVM unreachable
        }
    }

    #[cfg(test)]
    fn generate(&mut self) -> Result<X25519KeyPair, Self::Error> {
        RNG.with(|rng| {
            match rng.borrow_mut().generate() {
                Ok(pair) => Ok(pair),
                Err(never) => match never {}, // LLVM unreachable
            }
        })
    }
}

impl Generator<X25519KeyPair> for OsRng {
    type Error = Never;

    fn generate(&mut self) -> Result<X25519KeyPair, Self::Error> {
        let (publ, sec) = gen_x25519();
        let publ = X25519Public::from_slice(publ.as_ref()).expect("two types are equivalent");
        let sec = X25519Private::from_slice(sec.as_ref()).expect("two types are equivalent");
        Ok(X25519KeyPair::from_keypair(sec, publ))
    }
}

impl Generator<X25519KeyPair> for XorShiftRng {
    type Error = Never;

    fn generate(&mut self) -> Result<X25519KeyPair, Self::Error> {
        let (publ, sec) = gen_x25519();
        let publ = X25519Public::from_slice(publ.as_ref()).expect("two types are equivalent");
        let sec = X25519Private::from_slice(sec.as_ref()).expect("two types are equivalent");
        Ok(X25519KeyPair::from_keypair(sec, publ))
    }
}

impl Generator<SealKeyPair> for Random {
    type Error = ::std::io::Error;

    //FIXME: there is no distinction between the two generate functions
    #[cfg(not(test))]
    fn generate(&mut self) -> Result<SealKeyPair, Self::Error> {
        let mut rng = OsRng::new()?;
        match rng.generate() {
            Ok(pair) => Ok(pair),
            Err(never) => match never {}, // LLVM unreachable
        }
    }

    #[cfg(test)]
    fn generate(&mut self) -> Result<SealKeyPair, Self::Error> {
        RNG.with(|rng| {
            match rng.borrow_mut().generate() {
                Ok(pair) => Ok(pair),
                Err(never) => match never {}, // LLVM unreachable
            }
        })
    }
}

impl Generator<SealKeyPair> for OsRng {
    type Error = Never;

    fn generate(&mut self) -> Result<SealKeyPair, Self::Error> {
        let (publ, sec) = gen_curve25519xsalsa20poly1305();
        Ok(SealKeyPair::from_keypair(sec.into(), publ.into()))
    }
}

impl Generator<SealKeyPair> for XorShiftRng {
    type Error = Never;

    fn generate(&mut self) -> Result<SealKeyPair, Self::Error> {
        let (publ, sec) = gen_curve25519xsalsa20poly1305();
        Ok(SealKeyPair::from_keypair(sec.into(), publ.into()))
    }
}
