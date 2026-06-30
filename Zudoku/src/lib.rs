pub mod circuit;

#[cfg(test)]
mod flow_tests {
    use ark_bls12_381::{Bls12_381, Fr as BlsFr};
    use ark_groth16::Groth16;
    use ark_snark::{CircuitSpecificSetupSNARK, SNARK};
    use ark_std::{
        rand::{RngCore, SeedableRng},
        test_rng,
    };

    use crate::circuit::Puzzle;

    #[test]
    fn circuit_flow() {
        let sudoku = [
            [0, 0, 0, 2, 6, 0, 7, 0, 1],
            [6, 8, 0, 0, 7, 0, 0, 9, 0],
            [1, 9, 0, 0, 0, 4, 5, 0, 0],
            [8, 2, 0, 1, 0, 0, 0, 4, 0],
            [0, 0, 4, 6, 0, 2, 9, 0, 0],
            [0, 5, 0, 0, 0, 3, 0, 2, 8],
            [0, 0, 9, 3, 0, 0, 0, 7, 4],
            [0, 4, 0, 0, 5, 0, 0, 3, 6],
            [7, 0, 3, 0, 1, 8, 0, 0, 0],
        ];
        let solution = [
            [4, 3, 5, 2, 6, 9, 7, 8, 1],
            [6, 8, 2, 5, 7, 1, 4, 9, 3],
            [1, 9, 7, 8, 3, 4, 5, 6, 2],
            [8, 2, 6, 1, 9, 5, 3, 4, 7],
            [3, 7, 4, 6, 8, 2, 9, 1, 5],
            [9, 5, 1, 7, 4, 3, 6, 2, 8],
            [5, 1, 9, 3, 2, 6, 8, 7, 4],
            [2, 4, 8, 9, 5, 7, 1, 3, 6],
            [7, 6, 3, 4, 1, 8, 2, 5, 9],
        ];

        let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());
        let (pk, vk) = {
            let circuit = Puzzle::<9> {
                sudoku: None,
                solution: None,
            };
            Groth16::<Bls12_381>::setup(circuit, &mut rng).unwrap()
        };

        let proof = Groth16::<Bls12_381>::prove(
            &pk,
            Puzzle::<9> {
                sudoku: Some(sudoku),
                solution: Some(solution),
            },
            &mut rng,
        )
        .unwrap();

        let public_inputs: Vec<BlsFr> = sudoku
            .iter()
            .flat_map(|row| row.iter())
            .flat_map(|cell| (0..8).map(move |bit| BlsFr::from((cell >> bit) & 1)))
            .collect();

        assert!(Groth16::<Bls12_381>::verify(&vk, &public_inputs, &proof).unwrap());
    }
}
