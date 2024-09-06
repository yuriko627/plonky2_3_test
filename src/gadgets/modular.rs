use plonky2::hash::hash_types::RichField;
use plonky2::field::extension::Extendable;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};


// Extension of the CircuitBuilder trait
pub trait CircuitBuilderModular<F: RichField + Extendable<D>, const D: usize> {

    fn div_rem(
        &mut self,
        a: Target,
        b: Target,
    ) -> (Target, Target);

    // returns the reminder divided by b
    fn rem(&mut self, a: Target, b: Target) -> Target;
}

// Implement the CircuitBuilderModular trait
impl<F: RichField + Extendable<D>, const D: usize> CircuitBuilderModular<F, D> for CircuitBuilder<F, D>{

    fn div_rem(
        &mut self,
        a: Target,
        b: Target,
    ) -> (Target, Target) {

        // Create virtual targets for the quotient and remainder
        let div = self.add_virtual_target();



        // TODO: this is not correctly doing a modular reduction. fix this later 
        let mut pw = PartialWitness::new();
        let _ = pw.set_target(div, F::from_canonical_u32(1));
        let rem = self.sub(a, b);

        // perform division (a/b = div ... rem) by keep subtracting b from a until a is smaller than b

        // // Calculate div_b = div * b
        // let div_b = self.mul(div, b);

        // // Calculate div_b_plus_rem = div_b + rem
        // let div_b_plus_rem = self.add(div_b, rem);

        // // Ensure a = div_b_plus_rem
        // self.connect(a, div_b_plus_rem);

        // Ensure that rem is less than b by comparing them -> implement the comparison for Target by bitshift
        // let cmp_rem_b = self.cmp(&rem, b);

        // Assert that rem < b by ensuring cmp_rem_b == 1 (true)
        // self.assert_one(cmp_rem_b);

        (div, rem)
    }

    fn rem(&mut self, a: Target, b: Target) -> Target {
        let (_div, rem) = self.div_rem(a, b);
        rem
    }
}
