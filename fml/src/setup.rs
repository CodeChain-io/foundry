// Copyright 2020 Kodebox, Inc.
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

#[macro_export]
macro_rules! fml_setup {
    ($context: path, $preset: path, $debug: expr) => {
        type Context = fml::Context<$context>;
        #[cfg(feature = "single_process")]
        pub mod context {
            use super::*;
            use once_cell::sync::OnceCell;
            use std::collections::HashMap;
            use std::sync::Mutex;

            // We need to enclose the context in the Box so that it won't move.
            static POOL: OnceCell<Mutex<HashMap<fml::InstanceKey, Box<Context>>>> = OnceCell::new();
            pub fn get_context() -> &'static Context {
                let ptr: *const _ = &*POOL.get().unwrap().lock().unwrap().get(&fml::get_key()).unwrap();
                // TODO: Read the related section in Rustonomicon and make sure that this is safe.
                unsafe { &*ptr }
            }
            pub fn set_context(ctx: Context) {
                POOL.get_or_init(|| Default::default());
                let mut pool = POOL.get().unwrap().lock().unwrap();
                assert!(!pool.contains_key(&fml::get_key()));

                pool.insert(fml::get_key(), Box::new(ctx));
            }
            pub fn remove_context() {
                POOL.get().unwrap().lock().unwrap().remove(&fml::get_key()).unwrap();
            }
        }

        #[cfg(not(feature = "single_process"))]
        pub mod context {
            use super::*;
            static mut CONTEXT: Option<Context> = None;

            pub fn get_context() -> &'static Context {
                unsafe { CONTEXT.as_ref().unwrap() }
            }

            pub fn set_context(ctx: Context) {
                unsafe {
                    CONTEXT.replace(ctx);
                }
            }

            pub fn remove_context() {
                unsafe {
                    CONTEXT.take().unwrap();
                }
            }
        }

        pub fn get_context() -> &'static Context {
            context::get_context()
        }

        fn set_context(ctx: Context) {
            context::set_context(ctx)
        }

        fn remove_context() {
            context::remove_context()
        }

        #[cfg(feature = "single_process")]
        pub fn main_like(args: Vec<String>) {
            fml::run_control_loop::<cbsb::ipc::intra::Intra, $context, $preset>(args, Box::new(set_context), $debug);
            // be careful of the following order!
            fml::global::get().write().unwrap().no_drop = true;
            remove_context();
            fml::global::remove();
        }

        #[cfg(not(feature = "single_process"))]
        pub fn main_like(args: Vec<String>) {
            fml::run_control_loop::<cbsb::ipc::DefaultIpc, $context, $preset>(args, Box::new(set_context), $debug);
            // be careful of the following order!
            fml::global::get().write().unwrap().no_drop = true;
            remove_context();
            fml::global::remove();
        }
    };
}
