extern crate codechain_basesandbox as cbasesandbox;
use cbasesandbox::execution::executee;
use cbasesandbox::ipc::Ipc;

#[cfg(all(unix, target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    let ctx = executee::start::<cbasesandbox::execution::IpcUnixDomainSocket>();
    let r = ctx.ipc.recv();
    assert_eq!(r, b"Hello?\0");
    ctx.ipc.send(b"I'm here!\0");
    Ok(())
}
