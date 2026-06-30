use std::sync::Arc;

use ark_bls12_381::Bls12_381;
use ark_groth16::{ProvingKey, VerifyingKey};

#[derive(uniffi::Object)]
pub struct SudokuProvingKey(pub ProvingKey<Bls12_381>);

#[derive(uniffi::Object)]
pub struct SudokuVerifyingKey(pub VerifyingKey<Bls12_381>);

#[derive(Clone, uniffi::Record)]
pub struct ProvingMaterial {
    pub proving_key: Arc<SudokuProvingKey>,
    pub verifying_key: Arc<SudokuVerifyingKey>,
}
