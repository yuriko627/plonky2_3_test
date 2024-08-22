use std::fmt::Debug;
use std::marker::PhantomData;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, Field};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrix;

use p3_challenger::{HashChallenger, SerializingChallenger32};
use p3_circle::CirclePcs;
use p3_commit::ExtensionMmcs;
use p3_field::extension::BinomialExtensionField;
use p3_fri::FriConfig;
use p3_keccak::Keccak256Hash;
use p3_merkle_tree::FieldMerkleTreeMmcs;
use p3_mersenne_31::Mersenne31;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher32};
use p3_uni_stark::{prove, verify, StarkConfig};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use rand::{thread_rng, Rng};

// Define your AIR constraint inputs via declaring a Struct with relevent inputs in it
pub struct PolyAddAir {
	pub poly1: Vec<u32>,
	pub poly2: Vec<u32>,
	pub added_poly: Vec<u32>
}

// Define your Execution Trace Row size
// Air constraint is about working with the execution trace, where you can imagine a 2D matrix
// Each row represents the current iteration of the computation, and the colums for each row has elements that are associated with the computation
impl<F: Field> BaseAir<F> for PolyAddAir {
    fn width(&self) -> usize {
        10500 // poly1, pol2, added_poly's coeffs in total
    }
}

// Define your constraints
impl<AB: AirBuilder> Air<AB> for PolyAddAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let current = main.row_slice(0);
        // let next = main.row_slice(1);

        // Enforce input polynomial values and the added polynomial values
		for i in 0..3500 {
			builder.when_first_row().assert_eq(current[i], AB::Expr::from_canonical_u32(self.poly1[i]));
			builder.when_first_row().assert_eq(current[i+3500], AB::Expr::from_canonical_u32(self.poly2[i]));
            // builder.when_first_row().assert_eq(current[i+7000], AB::Expr::from_canonical_u32(self.added_poly[i]));
		}

    }
}

// Define a function to generate your program's execution trace
// This function keeps track of all the relevent state for each iteration, push them all into a 1D vector,
// and convert this 1D vector into a matrix in the dimension that matches your AIR script's width
pub fn generate_polyadd_trace<F: Field>(poly1:Vec<u32>, poly2:Vec<u32>) -> RowMajorMatrix<F> {
    // Declaring the total slots needed to keep track of the execution with the given parameter, which in this case, is num_steps multiply by 7000, where 7000 is the width of the AIR scripts.
    let mut values: Vec<F>= Vec::with_capacity(4 * 10500); // 4 is the minimum number of rows required

	// fill in the states in each iteration in the `values` vector
	for i in 0..3500 {
		values.push(F::from_canonical_u32(poly1[i]));
	}
	for i in 0..3500 {
		values.push(F::from_canonical_u32(poly2[i]));
	}

	// Add the 2 polynomials and push it to values vector
	for i in 0..3500 {
		values.push(F::from_canonical_u32((poly1[i] + poly2[i]) % 536870939));
	}

	// Fill in the rest of the slots (last 3 rows) with 0
	for _ in 0..(3 * 10500) {
		values.push(F::zero());
	}
    RowMajorMatrix::new(values, 10500)

}

fn main() -> Result<(), impl Debug> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    // Define zk system configuration
    type Val = Mersenne31;
    type Challenge = BinomialExtensionField<Val, 3>;

    type ByteHash = Keccak256Hash;
    type FieldHash = SerializingHasher32<ByteHash>;
    let byte_hash = ByteHash {};
    let field_hash = FieldHash::new(Keccak256Hash {});

    type MyCompress = CompressionFunctionFromHasher<u8, ByteHash, 2, 32>;
    let compress = MyCompress::new(byte_hash);

    type ValMmcs = FieldMerkleTreeMmcs<Val, u8, FieldHash, MyCompress, 32>;
    let val_mmcs = ValMmcs::new(field_hash, compress);

    type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    type Challenger = SerializingChallenger32<Val, HashChallenger<u8, ByteHash, 32>>;

    let fri_config = FriConfig {
        log_blowup: 1,
        num_queries: 100,
        proof_of_work_bits: 16,
        mmcs: challenge_mmcs,
    };

    type Pcs = CirclePcs<Val, ValMmcs, ChallengeMmcs>;
    let pcs = Pcs {
        mmcs: val_mmcs,
        fri_config,
        _phantom: PhantomData,
    };

    type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;
    let config = MyConfig::new(pcs);

	let mut rng = thread_rng();
	let random_poly1: Vec<u32> = (0..3500).map(|_| {
		rng.gen_range(0..536870939) // chose an arbitrary 30-bits prime number
	}).collect();

	let random_poly2: Vec<u32> = (0..3500).map(|_| {
		rng.gen_range(0..536870939)
	}).collect();

	let mut added_poly: Vec<u32> = Vec::with_capacity(3500);

	// Add the 2 polynomials
	for i in 0..3500 {
		added_poly.push((random_poly1[i] + random_poly2[i]) % 536870939);
	}

    let air = PolyAddAir { poly1: random_poly1.clone(), poly2: random_poly2.clone(), added_poly };

    let trace = generate_polyadd_trace::<Val>(random_poly1, random_poly2);

    let mut challenger: SerializingChallenger32<Mersenne31, HashChallenger<u8, Keccak256Hash, 32>> = Challenger::from_hasher(vec![], byte_hash);
    let proof = prove(&config, &air, &mut challenger, trace, &vec![]);

    let mut challenger = Challenger::from_hasher(vec![], byte_hash);
    verify(&config, &air, &mut challenger, &proof, &vec![])
}