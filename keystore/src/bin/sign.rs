use ckey::Ed25519Private as Private;

// FIXME: Use a safer way to receive the private key instead of CLI
fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(format!("Wrong number of arguments; Expected 2, but given {}", args.len() - 1))
    }

    let private = Private::from_slice(&hex::decode(&args[1]).map_err(|_| "Failed to parse private key".to_owned())?)
        .ok_or_else(|| "Invalid private key".to_owned())?;
    let data = hex::decode(&args[2]).map_err(|_| "Failed to parse data".to_owned())?;
    let signature: Vec<u8> = ckey::sign(&data, &private).as_ref().to_vec();

    println!("{}", hex::encode(signature));

    Ok(())
}
