use std::println;

use clap::Subcommand;
use xcm::{decode_xcm_message, XcmMessage};

#[derive(Subcommand)]
pub enum XcmCommands {
    /// Decode an XCM message string
    Decode {
        #[arg(short, long)]
        message: String,
    },
}

fn print_decode_message(xcm: XcmMessage) {
    println!("\n Decoding XCM Message");
    println!("------------------------- \n");

    println!("Version:");
    println!("  V{} \n", &xcm.version());

    println!("Instructions:");
    println!("  {}\n", &xcm.size());

    println!("Encoded Size:");
    println!("  {} bytes", xcm.bytes());

    for (index, instr) in xcm.instructions().iter().enumerate() {
        println!("\n");
        println!("[{index}] {}", instr.name);

        for (key, lines) in &instr.params {
            println!("      {}:", key);

            for line in lines {
                for sub_line in line.lines() {
                    println!("         - {}", sub_line.trim_end());
                }
            }
        }
    }
}

pub fn handle(action: XcmCommands) {
    match action {
        XcmCommands::Decode { message } => {
            let parsed_msg = message.strip_prefix("0x").unwrap_or(&message);
            match decode_xcm_message(parsed_msg.to_string()) {
                Ok(xcm) => {
                    println!("XCM Decoded, {xcm:?}");
                    print_decode_message(xcm);
                }
                Err(err) => {
                    println!("An error occured, {err:?}");
                }
            }
        }
    }
}
