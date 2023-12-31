use std::panic::panic_any;

use blstrs::Scalar as Fr;
use ff::Field;
use filecoin_proofs::{
    as_safe_commitment, verify_seal, DefaultOctLCTree, DefaultTreeDomain, PoRepConfig,
    SECTOR_SIZE_2_KIB, TEST_SEED,
};
use fr32::bytes_into_fr;
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use storage_proofs_core::{api_version::ApiVersion, sector::SectorId};

#[test]
fn test_verify_seal_fr32_validation() {
    let convertible_to_fr_bytes = [0; 32];
    let out = bytes_into_fr(&convertible_to_fr_bytes);
    assert!(out.is_ok(), "tripwire");

    let not_convertible_to_fr_bytes = [255; 32];
    let out = bytes_into_fr(&not_convertible_to_fr_bytes);
    assert!(out.is_err(), "tripwire");

    let arbitrary_porep_id = [87; 32];

    // Test failure for invalid comm_r conversion.
    {
        let result = verify_seal::<DefaultOctLCTree>(
            &PoRepConfig::new_groth16(SECTOR_SIZE_2_KIB, arbitrary_porep_id, ApiVersion::V1_1_0),
            not_convertible_to_fr_bytes,
            convertible_to_fr_bytes,
            [0; 32],
            SectorId::from(0),
            [0; 32],
            [0; 32],
            &[],
        );

        if let Err(err) = result {
            let needle = "Invalid all zero commitment";
            let haystack = format!("{}", err);

            assert!(
                haystack.contains(needle),
                "\"{}\" did not contain \"{}\"",
                haystack,
                needle,
            );
        } else {
            panic_any("should have failed comm_r to Fr32 conversion");
        }
    }

    // Test failure for invalid comm_d conversion.
    {
        let result = verify_seal::<DefaultOctLCTree>(
            &PoRepConfig::new_groth16(SECTOR_SIZE_2_KIB, arbitrary_porep_id, ApiVersion::V1_1_0),
            convertible_to_fr_bytes,
            not_convertible_to_fr_bytes,
            [0; 32],
            SectorId::from(0),
            [0; 32],
            [0; 32],
            &[],
        );

        if let Err(err) = result {
            let needle = "Invalid all zero commitment";
            let haystack = format!("{}", err);

            assert!(
                haystack.contains(needle),
                "\"{}\" did not contain \"{}\"",
                haystack,
                needle,
            );
        } else {
            panic_any("should have failed comm_d to Fr32 conversion");
        }
    }

    // Test failure for verifying an empty proof.
    {
        let non_zero_commitment_fr_bytes = [1; 32];
        let out = bytes_into_fr(&non_zero_commitment_fr_bytes);
        assert!(out.is_ok(), "tripwire");

        let result = verify_seal::<DefaultOctLCTree>(
            &PoRepConfig::new_groth16(SECTOR_SIZE_2_KIB, arbitrary_porep_id, ApiVersion::V1_1_0),
            non_zero_commitment_fr_bytes,
            non_zero_commitment_fr_bytes,
            [0; 32],
            SectorId::from(0),
            [0; 32],
            [0; 32],
            &[],
        );

        if let Err(err) = result {
            let needle = "Invalid proof bytes (empty vector)";
            let haystack = format!("{}", err);

            assert!(
                haystack.contains(needle),
                "\"{}\" did not contain \"{}\"",
                haystack,
                needle,
            );
        } else {
            panic_any("should have failed due to empty proof bytes");
        }
    }
}

#[test]
fn test_random_domain_element() {
    let mut rng = XorShiftRng::from_seed(TEST_SEED);

    for _ in 0..100 {
        let random_el: DefaultTreeDomain = Fr::random(&mut rng).into();
        let mut randomness = [0u8; 32];
        randomness.copy_from_slice(AsRef::<[u8]>::as_ref(&random_el));
        let back: DefaultTreeDomain =
            as_safe_commitment(&randomness, "test").expect("failed to get domain from randomness");
        assert_eq!(back, random_el);
    }
}
