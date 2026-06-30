use ark_ff::Field;
use ark_r1cs_std::prelude::AllocationMode;
use ark_r1cs_std::{
    prelude::{AllocVar, Boolean, EqGadget},
    uint8::UInt8,
};
use ark_r1cs_std::{R1CSVar, ToBitsGadget};
use ark_relations::r1cs::ConstraintSystem;
use ark_relations::r1cs::Namespace;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use std::borrow::Borrow;

pub struct Sudoku<const N: usize, ConstraintF: Field>([[UInt8<ConstraintF>; N]; N]);

pub struct Solution<const N: usize, ConstraintF: Field>([[UInt8<ConstraintF>; N]; N]);

pub struct Puzzle<const N: usize> {
    pub sudoku: Option<[[u8; N]; N]>,
    pub solution: Option<[[u8; N]; N]>,
}

pub fn check_rows<const N: usize, ConstraintF: Field>(
    solution: &Solution<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    for row in &solution.0 {
        for (j, cell) in row.iter().enumerate() {
            for prev in &row[0..j] {
                cell.is_neq(&prev)?.enforce_equal(&Boolean::TRUE)?;
            }
        }
    }
    Ok(())
}

pub fn check_cols<const N: usize, ConstraintF: Field>(
    solution: &Solution<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    let mut transpose: Vec<Vec<UInt8<ConstraintF>>> = Vec::with_capacity(N * N);
    for i in 0..9 {
        let col = &solution
            .0
            .clone()
            .into_iter()
            .map(|s| s.into_iter().nth(i).unwrap())
            .collect::<Vec<UInt8<ConstraintF>>>();
        transpose.push(col.to_vec());
    }
    for row in transpose {
        for (j, cell) in row.iter().enumerate() {
            for prev in &row[0..j] {
                cell.is_neq(&prev)?.enforce_equal(&Boolean::TRUE)?;
            }
        }
    }
    Ok(())
}

pub fn check_3_by_3<const N: usize, ConstraintF: Field>(
    solution: &Solution<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    let mut flat: Vec<UInt8<ConstraintF>> = Vec::with_capacity(N * N);
    for i in 0..3 {
        for j in 0..3 {
            flat.push(solution.0[i][j].clone());
        }
    }
    for (j, cell) in flat.iter().enumerate() {
        for prev in &flat[0..j] {
            cell.is_neq(&prev)?.enforce_equal(&Boolean::TRUE)?;
        }
    }
    Ok(())
}

pub fn check_sudoku_solution<const N: usize, ConstraintF: Field>(
    sudoku: &Sudoku<N, ConstraintF>,
    solution: &Solution<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    for i in 0..9 {
        for j in 0..9 {
            let a = &sudoku.0[i][j];
            let b = &solution.0[i][j];
            (a.is_eq(b)?.or(&a.is_eq(&UInt8::constant(0))?)?).enforce_equal(&Boolean::TRUE)?;

            b.is_leq(&UInt8::constant(N as u8))?
                .and(&b.is_geq(&UInt8::constant(1))?)?
                .enforce_equal(&Boolean::TRUE)?;
        }
    }
    Ok(())
}

pub fn check_helper<const N: usize, ConstraintF: Field>(
    sudoku: &[[u8; N]; N],
    solution: &[[u8; N]; N],
) {
    let cs = ConstraintSystem::<ConstraintF>::new_ref();
    let sudoku_var = Sudoku::new_input(cs.clone(), || Ok(sudoku)).unwrap();
    let solution_var = Solution::new_witness(cs.clone(), || Ok(solution)).unwrap();
    check_sudoku_solution(&sudoku_var, &solution_var).unwrap();
    check_rows(&solution_var).unwrap();
    check_cols(&solution_var).unwrap();
    check_3_by_3(&solution_var).unwrap();
    assert!(cs.is_satisfied().unwrap());
}

impl<const N: usize, F: Field> AllocVar<[[u8; N]; N], F> for Sudoku<N, F> {
    fn new_variable<T: Borrow<[[u8; N]; N]>>(
        cs: impl Into<Namespace<F>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let cs = cs.into();
        let row = [(); N].map(|_| UInt8::constant(0));
        let mut puzzle = Sudoku([(); N].map(|_| row.clone()));
        let value = f().map_or([[0; N]; N], |f| *f.borrow());
        for (i, row) in value.into_iter().enumerate() {
            for (j, cell) in row.into_iter().enumerate() {
                puzzle.0[i][j] = UInt8::new_variable(cs.clone(), || Ok(cell), mode)?;
            }
        }
        Ok(puzzle)
    }
}

impl<const N: usize, F: Field> AllocVar<[[u8; N]; N], F> for Solution<N, F> {
    fn new_variable<T: Borrow<[[u8; N]; N]>>(
        cs: impl Into<Namespace<F>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let cs = cs.into();
        let row = [(); N].map(|_| UInt8::constant(0));
        let mut solution = Solution([(); N].map(|_| row.clone()));
        let value = f().map_or([[0; N]; N], |f| *f.borrow());
        for (i, row) in value.into_iter().enumerate() {
            for (j, cell) in row.into_iter().enumerate() {
                solution.0[i][j] = UInt8::new_variable(cs.clone(), || Ok(cell), mode)?;
            }
        }
        Ok(solution)
    }
}

impl<const N: usize, F: Field> ConstraintSynthesizer<F> for Puzzle<N> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let sudoku = self.sudoku;
        let solution = self.solution;

        let sudoku_var = Sudoku::new_input(cs.clone(), || {
            sudoku.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let solution_var = Solution::new_witness(cs.clone(), || {
            solution.ok_or(SynthesisError::AssignmentMissing)
        })?;

        check_sudoku_solution(&sudoku_var, &solution_var)?;
        check_rows(&solution_var)?;
        check_cols(&solution_var)?;
        check_3_by_3(&solution_var)?;
        Ok(())
    }
}

pub trait CmpGadget<ConstraintF: Field>: R1CSVar<ConstraintF> + EqGadget<ConstraintF> {
    #[inline]
    fn is_geq(&self, other: &Self) -> Result<Boolean<ConstraintF>, SynthesisError> {
        // self >= other => self == other || self > other
        //               => !(self < other)
        self.is_lt(other).map(|b| b.not())
    }

    #[inline]
    fn is_leq(&self, other: &Self) -> Result<Boolean<ConstraintF>, SynthesisError> {
        // self <= other => self == other || self < other
        //               => self == other || other > self
        //               => self >= other
        other.is_geq(self)
    }

    #[inline]
    fn is_gt(&self, other: &Self) -> Result<Boolean<ConstraintF>, SynthesisError> {
        // self > other => !(self == other  || self < other)
        //              => !(self <= other)
        self.is_leq(other).map(|b| b.not())
    }

    fn is_lt(&self, other: &Self) -> Result<Boolean<ConstraintF>, SynthesisError>;
}

impl<ConstraintF: Field> CmpGadget<ConstraintF> for UInt8<ConstraintF> {
    fn is_lt(&self, other: &Self) -> Result<Boolean<ConstraintF>, SynthesisError> {
        // Determine the variable mode.
        if self.is_constant() && other.is_constant() {
            let self_value = self.value().unwrap();
            let other_value = other.value().unwrap();
            let result = Boolean::constant(self_value < other_value);
            Ok(result)
        } else {
            let diff_bits = self.xor(other)?.to_bits_be()?.into_iter();
            let mut result = Boolean::FALSE;
            let mut a_and_b_equal_so_far = Boolean::TRUE;
            let a_bits = self.to_bits_be()?;
            let b_bits = other.to_bits_be()?;
            for ((a_and_b_are_unequal, a), b) in diff_bits.zip(a_bits).zip(b_bits) {
                let a_is_lt_b = a.not().and(&b)?;
                let a_and_b_are_equal = a_and_b_are_unequal.not();
                result = result.or(&a_is_lt_b.and(&a_and_b_equal_so_far)?)?;
                a_and_b_equal_so_far = a_and_b_equal_so_far.and(&a_and_b_are_equal)?;
            }
            Ok(result)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::circuit::CmpGadget;
    use ark_bls12_381::Fr as Fp;
    use ark_r1cs_std::{
        prelude::{AllocVar, AllocationMode, Boolean, EqGadget},
        uint8::UInt8,
    };
    use ark_relations::r1cs::{ConstraintSystem, SynthesisMode};
    use itertools::Itertools;

    #[test]
    fn test_comparison_for_u8() {
        let modes = [
            AllocationMode::Constant,
            AllocationMode::Input,
            AllocationMode::Witness,
        ];
        for (a, a_mode) in (0..=u8::MAX).cartesian_product(modes) {
            for (b, b_mode) in (0..=u8::MAX).cartesian_product(modes) {
                let cs = ConstraintSystem::<Fp>::new_ref();
                cs.set_mode(SynthesisMode::Prove {
                    construct_matrices: true,
                });
                let a_var = UInt8::new_variable(cs.clone(), || Ok(a), a_mode).unwrap();
                let b_var = UInt8::new_variable(cs.clone(), || Ok(b), b_mode).unwrap();
                if a < b {
                    a_var
                        .is_lt(&b_var)
                        .unwrap()
                        .enforce_equal(&Boolean::TRUE)
                        .unwrap();
                    a_var
                        .is_leq(&b_var)
                        .unwrap()
                        .enforce_equal(&Boolean::TRUE)
                        .unwrap();
                    a_var
                        .is_gt(&b_var)
                        .unwrap()
                        .enforce_equal(&Boolean::FALSE)
                        .unwrap();
                    a_var
                        .is_geq(&b_var)
                        .unwrap()
                        .enforce_equal(&Boolean::FALSE)
                        .unwrap();
                } else if a == b {
                    a_var
                        .is_lt(&b_var)
                        .unwrap()
                        .enforce_equal(&Boolean::FALSE)
                        .unwrap();
                    a_var
                        .is_leq(&b_var)
                        .unwrap()
                        .enforce_equal(&Boolean::TRUE)
                        .unwrap();
                    a_var
                        .is_gt(&b_var)
                        .unwrap()
                        .enforce_equal(&Boolean::FALSE)
                        .unwrap();
                    a_var
                        .is_geq(&b_var)
                        .unwrap()
                        .enforce_equal(&Boolean::TRUE)
                        .unwrap();
                } else {
                    a_var
                        .is_lt(&b_var)
                        .unwrap()
                        .enforce_equal(&Boolean::FALSE)
                        .unwrap();
                    a_var
                        .is_leq(&b_var)
                        .unwrap()
                        .enforce_equal(&Boolean::FALSE)
                        .unwrap();
                    a_var
                        .is_gt(&b_var)
                        .unwrap()
                        .enforce_equal(&Boolean::TRUE)
                        .unwrap();
                    a_var
                        .is_geq(&b_var)
                        .unwrap()
                        .enforce_equal(&Boolean::TRUE)
                        .unwrap();
                }
                assert!(cs.is_satisfied().unwrap(), "a: {a}, b: {b}");
            }
        }
    }
}
