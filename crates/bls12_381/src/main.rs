use bls12_381::*;

fn main() {
    generate_g1_add_params();
    generate_g1_msm_params(2);

    generate_pairing_pairs(2, true);

    generate_mapped_g1_to_fp(5);
    generate_mapped_g2_to_fp2(3);
}
