use clap::Subcommand;

mod mapping;
mod msm;
mod points;
mod shared;
mod signatures;

#[derive(Subcommand)]
pub enum BlsCommands {
    /// Generate a random G1 point
    #[command(name = "random-g1")]
    RandomG1,

    /// Generate a random G2 point
    #[command(name = "random-g2")]
    RandomG2,

    /// Add two G1 points
    #[command(name = "g1-add")]
    G1Add {
        /// First point as full hex encoding or JSON with {"x":"0x...","y":"0x..."}
        #[arg(short = 'a', long = "point-a")]
        point_a: String,

        /// Second point as full hex encoding or JSON with {"x":"0x...","y":"0x..."}
        #[arg(short = 'b', long = "point-b")]
        point_b: String,
    },

    /// Generate deterministic G1-add test vectors
    #[command(name = "g1-add-testdata")]
    G1AddTestData {
        /// Number of test vectors to generate
        #[arg(short = 'n', long, default_value_t = 10)]
        count: usize,

        /// Write output to this file instead of stdout
        #[arg(short, long)]
        output_file: Option<String>,
    },

    /// Add two G2 points
    #[command(name = "g2-add")]
    G2Add {
        /// First point as full hex encoding or JSON with {"x":["0x...","0x..."],"y":["0x...","0x..."]}
        #[arg(short = 'a', long = "point-a")]
        point_a: String,

        /// Second point as full hex encoding or JSON with {"x":["0x...","0x..."],"y":["0x...","0x..."]}
        #[arg(short = 'b', long = "point-b")]
        point_b: String,
    },

    /// Generate deterministic G2-add test vectors
    #[command(name = "g2-add-testdata")]
    G2AddTestData {
        /// Number of test vectors to generate
        #[arg(short = 'n', long, default_value_t = 10)]
        count: usize,

        /// Write output to this file instead of stdout
        #[arg(short, long)]
        output_file: Option<String>,
    },

    /// Generate deterministic G1 MSM test data from k point-scalar pairs
    #[command(name = "g1-msm-testdata")]
    G1MsmTestData {
        /// Number of point-scalar pairs per test vector
        #[arg(short = 'k', long)]
        pairs: usize,

        /// Number of test vectors to generate (default: 1)
        #[arg(short = 'n', long, default_value_t = 1)]
        count: usize,

        /// Write output to this file instead of stdout
        #[arg(short, long)]
        output_file: Option<String>,
    },

    /// Generate deterministic G2 MSM test data from k point-scalar pairs
    #[command(name = "g2-msm-testdata")]
    G2MsmTestData {
        /// Number of point-scalar pairs per test vector
        #[arg(short = 'k', long)]
        pairs: usize,

        /// Number of test vectors to generate (default: 1)
        #[arg(short = 'n', long, default_value_t = 1)]
        count: usize,

        /// Write output to this file instead of stdout
        #[arg(short, long)]
        output_file: Option<String>,
    },

    /// Perform G1 MSM for the provided input data
    #[command(name = "g1-msm")]
    G1Msm {
        /// MSM data as full precompile hex input or JSON with {"points":[...],"scalars":[...]}
        #[arg(short, long)]
        data: String,
    },

    /// Perform G2 MSM for the provided input data
    #[command(name = "g2-msm")]
    G2Msm {
        /// MSM data as full precompile hex input or JSON with {"points":[...],"scalars":[...]}
        #[arg(short, long)]
        data: String,
    },

    /// Validate G1 MSM data without performing execution
    #[command(name = "g1-msm-validate")]
    G1MsmValidate {
        /// MSM data as full precompile hex input or JSON with {"points":[...],"scalars":[...]}
        #[arg(short, long)]
        data: String,
    },

    /// Validate G2 MSM data without performing execution
    #[command(name = "g2-msm-validate")]
    G2MsmValidate {
        /// MSM data as full precompile hex input or JSON with {"points":[...],"scalars":[...]}
        #[arg(short, long)]
        data: String,
    },

    /// Map Fp input to a G1 point
    #[command(name = "map-fp-to-g1")]
    MapFpToG1 {
        /// Fp input as 0x-hex (32 or 64 bytes) or JSON: {"value":"0x..."}
        #[arg(short, long)]
        fp: Option<String>,
    },

    /// Generate deterministic MapFp→G1 test vectors
    #[command(name = "map-fp-testdata")]
    MapFpTestData {
        /// Number of test vectors to generate
        #[arg(short = 'n', long, default_value_t = 10)]
        count: usize,

        /// Write output to this file instead of stdout
        #[arg(short, long)]
        output_file: Option<String>,
    },

    /// Map Fp2 input to a G2 point
    #[command(name = "map-fp2-to-g2")]
    MapFp2ToG2 {
        /// Fp2 input as 0x-hex (128 bytes) or JSON: {"value":["0x...","0x..."]}
        #[arg(short, long)]
        fp2: Option<String>,
    },

    /// Generate deterministic MapFp2→G2 test vectors
    #[command(name = "map-fp2-testdata")]
    MapFp2TestData {
        /// Number of test vectors to generate
        #[arg(short = 'n', long, default_value_t = 10)]
        count: usize,

        /// Write output to this file instead of stdout
        #[arg(short, long)]
        output_file: Option<String>,
    },

    /// Generate one BLS signature for a message
    Sign {
        /// Secret key scalar as decimal or 0x-hex (max 32 bytes)
        #[arg(short, long)]
        secret_key: String,

        /// Message to sign
        #[arg(short, long)]
        message: String,
    },

    /// Verify one BLS signature using pairing logic
    Verify {
        /// Signature as full G1 hex encoding or JSON point
        #[arg(short, long)]
        signature: String,

        /// Public key as full G2 hex encoding or JSON point
        #[arg(short, long)]
        pubkey: String,

        /// Message that was signed
        #[arg(short, long)]
        message: String,
    },

    /// Generate deterministic batch-signature test data
    #[command(name = "batch-sign-testdata")]
    BatchSignTestData {
        /// Number of signatures in the batch
        #[arg(short, long)]
        count: usize,

        /// Output format: 'hex', 'json', or 'both' (default: both)
        #[arg(short, long, default_value = "both")]
        output: String,
    },

    /// Aggregate multiple BLS signatures into one
    #[command(name = "batch-aggregate")]
    BatchAggregate {
        /// Signatures payload as concatenated hex or JSON {"signatures":[...G1 points...]}
        #[arg(short, long)]
        signatures: String,
    },

    /// Verify a batch of BLS signatures
    #[command(name = "batch-verify")]
    BatchVerify {
        /// JSON payload with messages, signatures, pubkeys, and optional aggregated_signature
        #[arg(short, long)]
        data: String,
    },

    /// End-to-end batch-signature smoke flow (generate -> aggregate -> verify)
    #[command(name = "batch-smoke")]
    BatchSmoke {
        /// Number of signatures in the batch
        #[arg(short, long, default_value_t = 4)]
        count: usize,

        /// Output format: 'summary' or 'json' (default: summary)
        #[arg(short, long, default_value = "summary")]
        output: String,
    },
}

pub fn handle(action: BlsCommands) {
    match action {
        BlsCommands::RandomG1 => points::cmd_random_g1(),
        BlsCommands::RandomG2 => points::cmd_random_g2(),
        BlsCommands::G1Add { point_a, point_b } => points::cmd_g1_add(point_a, point_b),
        BlsCommands::G1AddTestData { count, output_file } => {
            points::cmd_g1_add_testdata(count, output_file)
        }
        BlsCommands::G2Add { point_a, point_b } => points::cmd_g2_add(point_a, point_b),
        BlsCommands::G2AddTestData { count, output_file } => {
            points::cmd_g2_add_testdata(count, output_file)
        }
        BlsCommands::G1MsmTestData { pairs, count, output_file } => {
            msm::cmd_g1_msm_testdata(pairs, count, output_file)
        }
        BlsCommands::G2MsmTestData { pairs, count, output_file } => {
            msm::cmd_g2_msm_testdata(pairs, count, output_file)
        }
        BlsCommands::G1Msm { data } => msm::cmd_g1_msm(data),
        BlsCommands::G2Msm { data } => msm::cmd_g2_msm(data),
        BlsCommands::G1MsmValidate { data } => msm::cmd_g1_msm_validate(data),
        BlsCommands::G2MsmValidate { data } => msm::cmd_g2_msm_validate(data),
        BlsCommands::MapFpToG1 { fp } => mapping::cmd_map_fp_to_g1(fp),
        BlsCommands::MapFpTestData { count, output_file } => {
            mapping::cmd_map_fp_testdata(count, output_file)
        }
        BlsCommands::MapFp2ToG2 { fp2 } => mapping::cmd_map_fp2_to_g2(fp2),
        BlsCommands::MapFp2TestData { count, output_file } => {
            mapping::cmd_map_fp2_testdata(count, output_file)
        }
        BlsCommands::Sign { secret_key, message } => signatures::cmd_sign(secret_key, message),
        BlsCommands::Verify {
            signature,
            pubkey,
            message,
        } => signatures::cmd_verify(signature, pubkey, message),
        BlsCommands::BatchSignTestData { count, output } => {
            signatures::cmd_batch_sign_testdata(count, output)
        }
        BlsCommands::BatchAggregate { signatures } => signatures::cmd_batch_aggregate(signatures),
        BlsCommands::BatchVerify { data } => signatures::cmd_batch_verify(data),
        BlsCommands::BatchSmoke { count, output } => signatures::cmd_batch_smoke(count, output),
    }
}
