use clap::Subcommand;
use xcm::decode_xcm_message;

#[derive(Subcommand)]
pub enum XcmCommands {
    /// Decode an XCM message string
    Decode {
        #[arg(short, long)]
        message: String,
    },
}

fn print_decode_message(message: Vec<u8>, version: u32, size: usize) {
    println!("\n Decoding XCM Message");
    println!("------------------------- \n");

    println!("Version:");
    println!("  V{} \n", &version);

    println!("Instructions:");
    println!("  {size}\n");

    let xcm_encoded_size = message.len();
    println!("Encoded Size:");
    println!("  {xcm_encoded_size} bytes");
}

pub fn handle(action: XcmCommands) {
    match action {
        XcmCommands::Decode { message } => {
            let parsed_msg = message.strip_prefix("0x").unwrap_or(&message);
            match decode_xcm_message(parsed_msg.to_string()) {
                Ok(xcm) => {}
                Err(err) => {}
            }
        }
    }
}
