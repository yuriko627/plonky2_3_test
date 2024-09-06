use anyhow::Result;
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, Witness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::iop::target::Target;
use rand::Rng;
use plonky2_3_test::gadgets::modular::CircuitBuilderModular;

// proving addition of polynomials with 3500 terms each
fn main() -> Result<()> {
    use std::time::Instant;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // create the arithmetic circuit.
	let target_poly1 = builder.add_virtual_targets(3500);
    let target_poly2 = builder.add_virtual_targets(3500);
    let target_modulus = builder.add_virtual_target();
	let target_added_poly: Vec<Target> = target_poly1.clone().into_iter()
									.zip(target_poly2.clone())
									.map(|(l, r)| builder.add(l, r)).collect::<Vec<_>>()
                                    .into_iter()
                                    .map(|x| builder.rem(x, target_modulus))
                                    .collect();

    let num_gates = builder.num_gates();

    // Set all the target values as public inputs
    builder.register_public_inputs(&target_poly1);
    builder.register_public_inputs(&target_poly2);
    builder.register_public_input(target_modulus);
    builder.register_public_inputs(&target_added_poly);

    // build circuit once
    let data = builder.build::<C>();
    let num_public_inputs = data.common.num_public_inputs;
    println!("Number of gates/constraints: {}", num_gates);
    println!("Number of public inputs: {}", num_public_inputs);

    let mut pw = PartialWitness::new();

    // Assign the modulus value as witness
    let prime_mod = 536870939; // I just chose an arbitrary 30-bits prime number for testings
    let _ = pw.set_target(target_modulus, F::from_canonical_u32(prime_mod));

	// generate 2 random polynomials and set them as witness
	let mut rng = rand::thread_rng();
    for (i, coeff_target_poly1) in target_poly1.into_iter().enumerate() {

        let value = F::from_canonical_u32(rng.gen_range(0..prime_mod));
        // println!("Setting target_poly1[{}] to: {}", i, value);
        let _ = pw.set_target(coeff_target_poly1, value);

    }

    for coeff_target_poly2 in target_poly2.into_iter() {
        let _ = pw.set_target(coeff_target_poly2, F::from_canonical_u32(rng.gen_range(0..prime_mod)));
    }

    // generate proof
    let start_proving = Instant::now();
    let proof = data.prove(pw.clone()).unwrap();
    let end_proving = start_proving.elapsed();
    println!("proving time: {:.2?}", end_proving);


	for i in 0..10 { // just showing the first 10 polynomials addition
		println!(
			"Proved that we computed {} + {} = {}",
			proof.public_inputs[i], proof.public_inputs[i+3500], proof.public_inputs[i+7000]
		);
	}

    // verify proof
    let start_verify = Instant::now();
    let verify_result = data.verify(proof);
    let end_verify = start_verify.elapsed();
    println!("verification time: {:.2?}", end_verify);

    verify_result

}