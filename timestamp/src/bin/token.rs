fn main() {
    let args = std::env::args().collect();
    foundry_module_rt::start::<foundry_process_sandbox::ipc::unix_socket::DomainSocket, foundry_timestamp::token::Module>(
        args,
    );
}
