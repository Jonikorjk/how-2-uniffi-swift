use std::{sync::Arc};

use ark_bls12_381::{Bls12_381, Fr as BlsFr};
use ark_groth16::Groth16;
use ark_snark::{CircuitSpecificSetupSNARK, SNARK};
use ark_std::rand::{rngs::StdRng, SeedableRng};
use zudoku::circuit::Puzzle;

use crate::{
    io::{SudokuCircuitInput, SudokuProof},
    keys::{ProvingMaterial, SudokuProvingKey, SudokuVerifyingKey},
};

mod io;
mod keys;


uniffi::setup_scaffolding!();

const SUDOKU_SIZE: usize = 9;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ZudokuError {
    #[error("Setup error {message}")]
    Setup { message: String },
    #[error("Invalid input {message}")]
    InvalidInput { message: String },
    #[error("Proving error {message}")]
    Proving { message: String },
    #[error("Verification error {message}")]
    Verification { message: String },
}

#[derive(uniffi::Object)]
pub struct SudokuCircuit;

#[uniffi::export]
impl SudokuCircuit {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }
}

#[uniffi::export]
impl SudokuCircuit {
    pub fn setup(&self, seed: u64) -> Result<ProvingMaterial, ZudokuError> {
        let mut rng = StdRng::seed_from_u64(seed);
        let circuit = Puzzle::<SUDOKU_SIZE> {
            sudoku: None,
            solution: None,
        };

        let (pk, vk) =
            Groth16::<Bls12_381>::setup(circuit, &mut rng).map_err(|error| ZudokuError::Setup {
                message: format!("failed to set up proving material: {error}"),
            })?;

        Ok(ProvingMaterial {
            proving_key: Arc::new(SudokuProvingKey(pk)),
            verifying_key: Arc::new(SudokuVerifyingKey(vk)),
        })
    }

    pub fn generate_proof(
        &self,
        input: SudokuCircuitInput,
        proving_key: Arc<SudokuProvingKey>,
        seed: u64,
    ) -> Result<Arc<SudokuProof>, ZudokuError> {
        let puzzle = grid_into_array(input.puzzle, "puzzle")?;
        let solution = grid_into_array(input.solution, "solution")?;
        let mut rng = StdRng::seed_from_u64(seed);

        let circuit = Puzzle::<SUDOKU_SIZE> {
            sudoku: Some(puzzle),
            solution: Some(solution),
        };

        let proof =
            Groth16::<Bls12_381>::prove(&proving_key.0, circuit, &mut rng).map_err(|error| {
                ZudokuError::Proving {
                    message: format!("failed to generate proof: {error}"),
                }
            })?;

        Ok(Arc::new(SudokuProof(proof)))
    }

    pub fn verify_proof(
        &self,
        puzzle: Vec<Vec<u8>>,
        proof: Arc<SudokuProof>,
        verifying_key: Arc<SudokuVerifyingKey>,
    ) -> Result<bool, ZudokuError> {
        let puzzle = grid_into_array(puzzle, "puzzle")?;

        let public_inputs: Vec<BlsFr> = puzzle
            .iter()
            .flat_map(|row| row.iter())
            .flat_map(|cell| (0..8).map(move |bit| BlsFr::from((cell >> bit) & 1)))
            .collect();

        Groth16::<Bls12_381>::verify(&verifying_key.0, &public_inputs, &proof.0).map_err(|error| {
            ZudokuError::Verification {
                message: format!("failed to verify proof: {error}"),
            }
        })
    }
}

fn grid_into_array(
    rows: Vec<Vec<u8>>,
    name: &str,
) -> Result<[[u8; SUDOKU_SIZE]; SUDOKU_SIZE], ZudokuError> {
    if rows.len() != SUDOKU_SIZE {
        return Err(ZudokuError::InvalidInput {
            message: format!("{name} must contain exactly {SUDOKU_SIZE} rows"),
        });
    }

    let mut grid = [[0u8; SUDOKU_SIZE]; SUDOKU_SIZE];
    for (row_index, row) in rows.into_iter().enumerate() {
        if row.len() != SUDOKU_SIZE {
            return Err(ZudokuError::InvalidInput {
                message: format!("{name} row {row_index} must contain exactly {SUDOKU_SIZE} cells"),
            });
        }

        for (column_index, value) in row.into_iter().enumerate() {
            if value > SUDOKU_SIZE as u8 {
                return Err(ZudokuError::InvalidInput {
                    message: format!(
                        "{name} cell [{row_index}][{column_index}] must be between 0 and {SUDOKU_SIZE}"
                    ),
                });
            }
            grid[row_index][column_index] = value;
        }
    }

    Ok(grid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn puzzle() -> Vec<Vec<u8>> {
        vec![
            vec![0, 0, 0, 2, 6, 0, 7, 0, 1],
            vec![6, 8, 0, 0, 7, 0, 0, 9, 0],
            vec![1, 9, 0, 0, 0, 4, 5, 0, 0],
            vec![8, 2, 0, 1, 0, 0, 0, 4, 0],
            vec![0, 0, 4, 6, 0, 2, 9, 0, 0],
            vec![0, 5, 0, 0, 0, 3, 0, 2, 8],
            vec![0, 0, 9, 3, 0, 0, 0, 7, 4],
            vec![0, 4, 0, 0, 5, 0, 0, 3, 6],
            vec![7, 0, 3, 0, 1, 8, 0, 0, 0],
        ]
    }

    fn solution() -> Vec<Vec<u8>> {
        vec![
            vec![4, 3, 5, 2, 6, 9, 7, 8, 1],
            vec![6, 8, 2, 5, 7, 1, 4, 9, 3],
            vec![1, 9, 7, 8, 3, 4, 5, 6, 2],
            vec![8, 2, 6, 1, 9, 5, 3, 4, 7],
            vec![3, 7, 4, 6, 8, 2, 9, 1, 5],
            vec![9, 5, 1, 7, 4, 3, 6, 2, 8],
            vec![5, 1, 9, 3, 2, 6, 8, 7, 4],
            vec![2, 4, 8, 9, 5, 7, 1, 3, 6],
            vec![7, 6, 3, 4, 1, 8, 2, 5, 9],
        ]
    }

    #[test]
    fn generates_and_verifies_proof() {
        let circuit = SudokuCircuit::new();
        let material = circuit.setup(7).unwrap();
        let proof = circuit
            .generate_proof(
                SudokuCircuitInput {
                    puzzle: puzzle(),
                    solution: solution(),
                },
                material.proving_key.clone(),
                11,
            )
            .unwrap();

        assert!(circuit
            .verify_proof(puzzle(), proof, material.verifying_key.clone())
            .unwrap());
    }
}
