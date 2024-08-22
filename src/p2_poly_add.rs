use anyhow::Result;
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::iop::target::Target;
// use plonky2_ecdsa::gadgets::biguint::CircuitBuilderBiguint;
use rand::Rng;

// proving addition of polynomial with 3500 terms each
fn main() -> Result<()> {
    use std::time::Instant;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // The arithmetic circuit.
	let poly1 = builder.add_virtual_targets(3500);
    let poly2 = builder.add_virtual_targets(3500);
	let added_poly: Vec<Target> = poly1.clone().into_iter()
									.zip(poly2.clone())
									.map(|(l, r)| builder.add(l, r))
									.collect();

    // Public inputs are the two input polynomials and the computed outpout polynomial.
    builder.register_public_inputs(&poly1);
    builder.register_public_inputs(&poly2);
    builder.register_public_inputs(&added_poly);

    // Provide initial values.
    let mut pw = PartialWitness::new();

	// generate 2 random polynomials
	let mut rng = rand::thread_rng();
    for coeff_poly1 in poly1.into_iter() {
        let _ = pw.set_target(coeff_poly1, F::from_canonical_u32(rng.gen_range(0..536870939))); // chose an arbitrary 30-bits prime number
    }

    for coeff_poly2 in poly2.into_iter() {
        let _ = pw.set_target(coeff_poly2, F::from_canonical_u32(rng.gen_range(0..536870939)));
    }

    let data = builder.build::<C>();

    let now = Instant::now();
    let proof = data.prove(pw)?;
    let elapsed = now.elapsed();
    println!("proving time: {:.2?}", elapsed);

	for i in 0..5 { // just showing the first 5 coefficients of the polynomials addition
		println!(
			"Proved that we computed {} + {} = {}",
			proof.public_inputs[i], proof.public_inputs[i+3500], proof.public_inputs[i+7000]
		);
	}

    // Measure verification time
    let verify_start = Instant::now();
    let verify_result = data.verify(proof);
    let verify_elapsed = verify_start.elapsed();
    println!("verification time: {:.2?}", verify_elapsed);

    verify_result

}