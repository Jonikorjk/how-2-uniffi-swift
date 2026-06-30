import Foundation
import Testing
import SudokuBindings
@testable import SudokuCircuit

/// Tests the UniFFI-bridged zero-knowledge circuit for 9×9 Sudoku puzzles.
///
/// The circuit supports any correctly formatted 9×9 Sudoku:
///
/// - `puzzle` contains the original puzzle.
/// - `0` represents an empty cell.
/// - Values from `1...9` represent the given clues.
/// - `solution` contains the completed solution.
/// - Every row, column, and 3×3 box must contain the values `1...9`.
/// - The solution must match all clues from the original puzzle.
///
/// Test flow:
///
/// 1. `setup(seed:)` generates the proving and verifying keys.
/// 2. `generateProof(...)` creates a zero-knowledge proof for the solution.
/// 3. The solution remains private and is used as the witness.
/// 4. The original puzzle is used as the public input.
/// 5. `verifyProof(...)` confirms that a valid solution exists without receiving
///    or revealing the solution itself.
///
/// The circuit does not solve the Sudoku puzzle. A completed solution must be
/// provided. The circuit proves that the supplied solution is valid without
/// exposing it.
///
/// The circuit supports only 9×9 puzzles because its size is fixed in Rust:
///
///     const SUDOKU_SIZE: usize = 9;
///
/// The proving and verifying keys can be reused for different 9×9 puzzles that
/// use the same circuit structure.
///
/// Fixed seeds make this test reproducible. Production code should use
/// cryptographically secure random seeds.
struct SudokuCircuitTests {

    @Test func testCircuitFlow() throws {
        let circuit = SudokuCircuit()
        let puzzle: [Data] = [
            Data([0, 0, 0, 2, 6, 0, 7, 0, 1]),
            Data([6, 8, 0, 0, 7, 0, 0, 9, 0]),
            Data([1, 9, 0, 0, 0, 4, 5, 0, 0]),
            Data([8, 2, 0, 1, 0, 0, 0, 4, 0]),
            Data([0, 0, 4, 6, 0, 2, 9, 0, 0]),
            Data([0, 5, 0, 0, 0, 3, 0, 2, 8]),
            Data([0, 0, 9, 3, 0, 0, 0, 7, 4]),
            Data([0, 4, 0, 0, 5, 0, 0, 3, 6]),
            Data([7, 0, 3, 0, 1, 8, 0, 0, 0]),
        ]
        let solution: [Data] = [
            Data([4, 3, 5, 2, 6, 9, 7, 8, 1]),
            Data([6, 8, 2, 5, 7, 1, 4, 9, 3]),
            Data([1, 9, 7, 8, 3, 4, 5, 6, 2]),
            Data([8, 2, 6, 1, 9, 5, 3, 4, 7]),
            Data([3, 7, 4, 6, 8, 2, 9, 1, 5]),
            Data([9, 5, 1, 7, 4, 3, 6, 2, 8]),
            Data([5, 1, 9, 3, 2, 6, 8, 7, 4]),
            Data([2, 4, 8, 9, 5, 7, 1, 3, 6]),
            Data([7, 6, 3, 4, 1, 8, 2, 5, 9]),
        ]

        let material = try circuit.setup(seed: 7)
        let proof = try circuit.generateProof(
            input: SudokuCircuitInput(puzzle: puzzle, solution: solution),
            provingKey: material.provingKey,
            seed: 11
        )
        let isValid = try circuit.verifyProof(
            puzzle: puzzle,
            proof: proof,
            verifyingKey: material.verifyingKey
        )

        #expect(isValid)

        // Visualized Solution of Sudoku in tests
        //  ┌───────┬───────┬───────┐
        //  │ 4 3 5 │ 2 6 9 │ 7 8 1 │
        //  │ 6 8 2 │ 5 7 1 │ 4 9 3 │
        //  │ 1 9 7 │ 8 3 4 │ 5 6 2 │
        //  ├───────┼───────┼───────┤
        //  │ 8 2 6 │ 1 9 5 │ 3 4 7 │
        //  │ 3 7 4 │ 6 8 2 │ 9 1 5 │
        //  │ 9 5 1 │ 7 4 3 │ 6 2 8 │
        //  ├───────┼───────┼───────┤
        //  │ 5 1 9 │ 3 2 6 │ 8 7 4 │
        //  │ 2 4 8 │ 9 5 7 │ 1 3 6 │
        //  │ 7 6 3 │ 4 1 8 │ 2 5 9 │
        //  └───────┴───────┴───────┘
    }
}
