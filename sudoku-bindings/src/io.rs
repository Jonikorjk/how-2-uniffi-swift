use ark_bls12_381::Bls12_381;
use ark_groth16::Proof;

#[derive(Clone, Debug, uniffi::Record)]
pub struct SudokuCircuitInput {
    pub puzzle: Vec<Vec<u8>>,
    pub solution: Vec<Vec<u8>>,
}

#[derive(uniffi::Object)]
pub struct SudokuProof(pub Proof<Bls12_381>);
