# The art of bridging Rust to Swift

If you have worked with niche products, you have probably encountered situations where some required functionality was not properly implemented in Swift. In cryptography, for example, you may find that a particular cryptographic algorithm does not have a Swift implementation. In such cases, you have the option of bridging existing logic from other programming languages. One of the most popular approaches is to bridge a C library. However, in this case, you have to operate with very low-level concepts, such as pointers and manual memory management, which complicates the development process. I guess anyone who has encountered a task that required bridging a C library knows what I am talking about. 

What is our salvation? - uniffi-rs. 

uniffi-rs is a multi-language bindings generator for Rust. In modern products, it is a good practice to write shared logic in Rust and bridge it between different platforms instead of implementing the same logic separately for each language.
In this article, we will first go through the main UniFFI API and understand its basic concepts. After that, we will consider a real example to see how and where UniFFI can be used in practice.

# Primary API

### `uniffi::Record`

`uniffi::Record` is similar to a Swift structure. It is used for simple data containers transferred by value between Rust and Swift.
The fields of a record are available directly from Swift.

```rust
#[derive(Clone, uniffi::Record)]
pub struct User {
    pub name: String,
    pub age: u32,
}
```

Swift usage:

```swift
let user = User(
    name: "John",
    age: 28
)

print(user.name)
print(user.age)
```

### `uniffi::Object` and `uniffi::constructor`

`uniffi::Object` is similar to a Swift class. The actual object remains managed by Rust, while Swift holds an opaque reference to it. Unlike a record, Swift cannot access the internal fields of a Rust object directly. It does not matter whether those fields are public or private in Rust. To interact with them, we must expose methods using `#[uniffi::export]`. `uniffi::constructor` exposes a Rust associated function as a Swift initializer.

```rust
#[derive(uniffi::Object)]
pub struct Calculator {
    multiplier: u32,
}

#[uniffi::export]
impl Calculator {
    #[uniffi::constructor]
    pub fn new(multiplier: u32) -> Self {
        Self { multiplier }
    }

    pub fn multiply(&self, value: u32) -> u32 {
        value * self.multiplier
    }

    pub fn multiplier(&self) -> u32 {
        self.multiplier
    }
}
```

Swift usage

```swift
let calculator = Calculator(multiplier: 10)

let result = calculator.multiply(value: 5)
let multiplier = calculator.multiplier()

print(result)
print(multiplier)

// Swift cannot access the internal multiplier field directly:

print(calculator.multiplier) // Not available
calculator.multiplier = 20 // Not available
```

All interaction with internal Rust fields must go through exported methods.
A constructor is not required when an object is created internally by Rust and returned from another function. However, if Swift needs to create it directly, the object must provide a function marked with `#[uniffi::constructor]`.
The simplest way to remember the difference is:

> `uniffi::Record` is like a Swift structure and exposes its fields. `uniffi::Object` is like a Swift class, but its internal Rust fields are hidden and accessible only through exported methods.

### Record containing an Object

A `uniffi::Record` can contain a reference to a `uniffi::Object`.
The record is transferred by value, but the object stored inside it remains managed by Rust. Swift receives an opaque reference to the object. To pass `uniffi::Object` to the `uniffi::Record` field use `Arc<_>`.

```rust
use std::sync::Arc;

#[derive(uniffi::Object)]
pub struct Account {
    identifier: u64,
}

#[uniffi::export]
impl Account {
    #[uniffi::constructor]
    pub fn new(identifier: u64) -> Self {
        Self { identifier }
    }
    
    pub fn identifier(&self) -> u64 {
        self.identifier
    }
}

#[derive(Clone, uniffi::Record)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub account: Arc<Account>, // Object in Record
}
```

Swift usage:

```swift
let account = Account(identifier: 42)

let user = User(
    name: "John",
    age: 28,
    account: account
)

print(user.name)
print(user.age)
print(user.account.identifier())
```

### `uniffi::export`

`#[uniffi::export]` tells `UniFFI` that a `Rust` function, implementation block, or trait must be included in the generated bindings. Without this attribute, the code remains available only inside `Rust` and will not be visible from `Swift`.

### Exporting an external function

The attribute can be placed on a free-standing Rust function:

```rust
#[uniffi::export]
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}
```

Swift usage:

```swift
let result = add(left: 10, right: 20)
print(result)
```

Free-standing functions are useful when an operation does not belong to a particular object and does not require any stored state.

#### Exporting an `impl` block

The attribute can also be placed on an `impl` block to expose its methods:

```rust
#[derive(uniffi::Object)]
pub struct Calculator {
    multiplier: u32,
}

#[uniffi::export]
impl Calculator {
    #[uniffi::constructor]
    pub fn new(multiplier: u32) -> Self {
        Self { multiplier }
    }
    
    pub fn multiply(&self, value: u32) -> u32 {
        value * self.multiplier
    }
}
```

Swift usage:

```swift
let calculator = Calculator(multiplier: 10)
let result = calculator.multiply(value: 5)
```

Only methods inside the exported impl block become available from Swift.

#### Exporting a trait

`#[uniffi::export(with_foreign)]` can also be used with Rust traits. An exported Rust trait is represented as a protocol in Swift.

```rust
#[uniffi::export(with_foreign)]
pub trait DoTrait: Send + Sync {
    fn do_stuff(&self);
}
```

The generated `Swift API` will contain a protocol similar to this:

```swift
protocol DoTrait {
    func doStuff()
}
```

The `with_foreign` option means that the trait can be implemented on the `Swift` side:

```swift
final class SwiftDoer: DoTrait {
    func doStuff() {
        print("Doing stuff in Swift")
    }
}
```

This `Swift` implementation can then be passed to `Rust`, allowing `Rust` code to call a method implemented in `Swift`.

Exported traits do not support generic type parameters, generic methods, or associated types. For example, the following traits cannot be exported directly:

```rust
// Is not supported
pub trait Storage<T> {
    fn save(&self, value: T);
}

// Is not supported
pub trait Storage {
    type Value;

    fn save(&self, value: Self::Value);
}
```

Instead, you must replace generic or associated types with concrete UniFFI-compatible types:

```rust
#[uniffi::export(with_foreign)]
pub trait StringStorage: Send + Sync {
    fn save(&self, value: String);
}
```

UniFFI needs to generate a concrete API for every target language. Generic parameters and associated Rust types do not provide a single concrete representation that can be transferred across the FFI boundary.

### `uniffi::Enum`

`uniffi::Enum` is similar to a Swift enum. It is used when a value can have one of several predefined states.

#### Simple enum

A simple enum contains variants without additional data.

```rust
#[derive(Clone, uniffi::Enum)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}
```

Swift usage:

```swift
let state = ConnectionState.connected

switch state {
case .disconnected:
    print("Disconnected")
case .connecting:
    print("Connecting")
case .connected:
    print("Connected")
}
```

#### Enum with associated data

A `UniFFI` enum can also store additional data inside its variants.

```rust
#[derive(Clone, uniffi::Enum)]
pub enum LoadingState {
    Idle,
    Loading(String),
    Success {
        value: String,
    },
    Failure {
        message: String,
        code: u32,
    },
}

```
Swift usage:
```swift
let state = LoadingState.success(
    value: "Loaded data"
)

switch state {
case .idle:
    print("Idle")
case .loading(let str):
    print("Loading" + str)
case .success(let value):
    print(value)
case .failure(let message, let code):
    print("\(message), code: \(code)")
}
```

All associated values must use UniFFI-compatible types.

### Returning an enum

Enums can be used as function arguments and return values:

```rust
#[uniffi::export]
pub fn load_data(
    should_succeed: bool,
) -> LoadingState {
    if should_succeed {
        LoadingState::Success {
            value: "Loaded data".to_string(),
        }
    } else {
        LoadingState::Failure {
            message: "Unable to load data".to_string(),
            code: 500,
        }
    }
}
```

Swift usage:

```swift
let state = loadData(shouldSucceed: true)

switch state {
case .success(let value):
    print(value)
case .failure(let message, let code):
    print("\(message), \(code)")
default:
    break

}
```

#### Enum containing an Object

A `uniffi::Enum` can contain a reference to a `uniffi::Object` as an associated value.
The enum is transferred by value, while the object remains managed by Rust.

```rust
use std::sync::Arc;

#[derive(uniffi::Object)]
pub struct Account {
    identifier: u64,
}

#[uniffi::export]
impl Account {
    #[uniffi::constructor]
    pub fn new(identifier: u64) -> Self {
        Self { identifier }
    }
    
    pub fn identifier(&self) -> u64 {
        self.identifier
    }
}

#[derive(Clone, uniffi::Enum)]
pub enum AccountResult {
    Success {
        account: Arc<Account>,
    },
    Failure {
        message: String,
    },
}
```

An exported function can return the enum:

```rust
#[uniffi::export]
pub fn find_account(
    identifier: u64,
) -> AccountResult {
    if identifier > 0 {
        AccountResult::Success {
            account: Arc::new(
                Account { identifier }
            ),
        }
    } else {
        AccountResult::Failure {
            message: "Invalid identifier".to_string(),
        }
    }
}
```

Swift usage:
```swift
let result = findAccount(identifier: 42)

switch result {
case .success(let account):
    print(account.identifier())
case .failure(let message):
    print(message)
}
```

### `Option`

UniFFI supports Rust's `Option<T>` when T is a UniFFI-compatible type. `Option<T>` is represented as an optional value in Swift.

```rust
#[uniffi::export]
pub fn find_name(
    identifier: u64,
) -> Option<String> {
    if identifier == 1 {
        Some("John".to_string())
    } else {
        None
    }
}
```

Swift usage:

```swift
let name = findName(identifier: 1)
if let name {
    print(name)
}
```

### `Result`

UniFFI supports Rust's `Result<T, E>` in exported functions and methods.
A Rust Result becomes a throwing function in Swift.
The error type must be exposed using `uniffi::Error`:

```rust
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum UserError {
    #[error("User was not found")]
    NotFound,
    #[error("Invalid identifier")]
    InvalidIdentifier,
}
```

The error can then be returned from an exported function:

```rust
#[uniffi::export]
pub fn get_name(
    identifier: u64,
) -> Result<String, UserError> {
    if identifier == 0 {
        return Err(
            UserError::InvalidIdentifier
        );
    }

    if identifier != 1 {
        return Err(UserError::NotFound);
    }

    Ok("John".to_string())
}
```

Swift usage:

```swift
do {
    let name = try getName(identifier: 1)
    print(name)
} catch {
    print(error)
}
```

#### Returning a result as a value

Sometimes success and failure are both expected states, and we do not want the Swift function to throw an error. In this case, we can define a `uniffi::Enum`:

```rust
#[derive(Clone, uniffi::Enum)]
pub enum OperationResult {
    Success {
        value: String,
    },
    Failure {
        message: String,
    },
}
```

Return it as a regular value:

```rust
#[uniffi::export]
pub fn perform_operation(
    should_succeed: bool,
) -> OperationResult {
    if should_succeed {
        OperationResult::Success {
            value: "Completed".to_string(),
        }
    } else {
        OperationResult::Failure {
            message: "Operation failed".to_string(),
        }
    }
}
```

Swift usage:

```swift
let result = performOperation(
    shouldSucceed: true
)

switch result {
case .success(let value):
    print(value)
case .failure(let message):
    print(message)
}
```

### `uniffi::custom_type!`

`uniffi::custom_type!` allows us to expose a Rust type through another type that UniFFI already supports.
For example, Uuid is not supported by UniFFI out of the box. However, we can transfer it across the FFI boundary as a String.
Register Uuid as a custom type:

```rust
uniffi::custom_type!(Uuid, String, {
    remote,
    lower: |uuid| uuid.to_string(),
    try_lift: |value| {
        Uuid::parse_str(&value).map_err(Into::into)
    },
});
```

The remote marker is required because Uuid is declared in another crate. lower converts the Rust value into the UniFFI-compatible bridge type:

```rust
lower: |uuid| uuid.to_string()
```

The conversion direction is:
Rust Uuid -> String -> Swift String
try_lift performs the reverse conversion:

```rust
try_lift: |value| {
    Uuid::parse_str(&value)
        .map_err(Into::into)
}
```

The conversion direction is: Swift String -> Rust String -> Rust Uuid.

The conversion is fallible because Swift may provide a string that is not a valid UUID.
After registering the custom type, Rust can use Uuid in exported functions, records, and enums. Swift will see these values as regular strings.


## A real-world example: zero-knowledge Sudoku

> [This repository](https://github.com/Jonikorjk/how-2-uniffi-swift) contains the complete source code, binding-generation instructions, and test setup. This section focuses primarily on explaining how the integration process works.

Lets consider that you want to prove knowledge of a solution to this Sudoku puzzle without revealing the solution:
This problem can be solved using a zkSNARK (Zero-Knowledge Succinct Non-Interactive). The Sudoku puzzle is treated as public data, while the solution remains private. The prover generates a cryptographic proof showing that the hidden solution satisfies all Sudoku rules. The verifier can validate the proof without ever learning the actual solution. This allows us to prove knowledge of a valid solution while preserving complete privacy.
The rules satisfier in another words circuit was written in Rust and developer of this curcuit gives you the following code with words:

> There are curcuit tests for 9x9 sudoku puzzle, your task is `setup`, `generate` and `verify` Sudoku solution on iOS side.


```rust
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
```

You notice that proof generation uses `Groth16` with a somewhat mysterious generic parameter: `Bls12_381`. Curious, you ask your colleague what it means. After a long conversation, he explains that `Groth16` is a `zkSNARK` proving system, while `Bls12_381` is the elliptic curve on which all cryptographic operations are performed.
At a high level, you can think of `Groth16` as the protocol that defines how proofs are generated and verified, while `Bls12_381` provides the mathematical foundation that makes those operations secure and efficient.

In other words:

* `Groth16` answers how we generate and verify proofs.
* `Bls12_381` defines where the underlying cryptographic computations take place.


Alright… The next your thing will be:
I need to import `Groth16` implementation on `iOS`, set `Bls12_381` and invoke prove and verify methods. You are starting to search `SwiftyGroth16` in `GitHub` and what? In best case you find a package that is not supported, or it exists but doesn't support the curve that you are using in curcuit, or nothing.

So now you have two options:

* Implement `Groth16` yourself in Swift.
* Reuse the existing Rust implementation and expose it to Swift.

Unless you have several months to spare and a PhD in cryptography, the second option sounds much more attractive.
This is exactly where UniFFI enters the picture.

### Bridging the circuit API

I recommend using cargo-swift, since it provides commands for initializing and packaging a Rust library as a Swift Package for iOS and macOS applications.
Install it with:
```bash
cargo install cargo-swift
```
To initialize the bindings crate, execute:
```bash
cargo swift init sudoku-bindings
```
From the original circuit flow, we need to expose three operations:
* `setup()`
* `generate_proof()`
* `verify_proof()`

> To successfully bridge code you should make uniffi compatible its input and output.

For example, consider the setup operation:

```rust

let (pk, vk) = Groth16::<Bls12_381>::setup(
    circuit,
    &mut rng,
).unwrap();
```
It returns two Arkworks types:
* `ProvingKey<Bls12_381>`
* `VerifyingKey<Bls12_381>`

UniFFI cannot expose these types directly. Therefore, we need to somehow bridge it.

One of the most easier practice is create a `uniffi::Object` wrapper around external type. In such case you skip `serialize` and `deserialize` logic, and it's easier than create a custom type.


```rust
#[derive(uniffi::Object)]
pub struct SudokuProvingKey(
    pub ProvingKey<Bls12_381>,
);

#[derive(uniffi::Object)]
pub struct SudokuVerifyingKey(
    pub VerifyingKey<Bls12_381>,
);
```

Since `uniffi-rs` does not support tuples in response type we should create a `ProvingMaterial` container. 

``` rust
#[derive(Clone, uniffi::Record)]
pub struct ProvingMaterial {
    pub proving_key: Arc<SudokuProvingKey>,
    pub verifying_key: Arc<SudokuVerifyingKey>,
}
```

And we are ready to bridge `setup()` function. To hold code clear lets declare `SudokuCircuit` object which will be like namespace instead of creating global functions.

``` rust
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
}
```

For the `generate_proof` and `verify_proof` stages we need to add `SudokuProof` and `SudokuCurcuitInput`.

``` rust
#[derive(Clone, Debug, uniffi::Record)]
pub struct SudokuCircuitInput {
    pub puzzle: Vec<Vec<u8>>,
    pub solution: Vec<Vec<u8>>,
}

#[derive(uniffi::Object)]
pub struct SudokuProof(pub Proof<Bls12_381>);
```

The bridged version of `generate_proof`:

``` rust
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
```

The bridged version of `verify_proof`: 

``` rust 
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
```

Finally, we are ready to generate bindings.

### Generate Bindings

`cargo swift` supports the following platforms:

```text
macos, ios, tvos, watchos, visionos, maccatalyst
```

Before generating the bindings, make sure that your UniFFI and `cargo-swift` versions are compatible. Currently, `cargo-swift` does not detect the UniFFI version used by your project automatically.

For example:

```text
UniFFI 0.30   -> cargo-swift 0.10
UniFFI 0.31   -> cargo-swift 0.11
UniFFI 0.31.1 -> cargo-swift 0.11.1
```

You can install a specific version using:

```bash
cargo install cargo-swift@0.11.1 -f
```

The complete compatibility table is available in the [`cargo-swift` repository](https://github.com/antoniusnaumann/cargo-swift#installing-for-a-different-uniffi-version).

If your application supports an older iOS version, make sure to pass `IPHONEOS_DEPLOYMENT_TARGET` when building the bindings. Otherwise, some dependencies may be built for a newer iOS version, producing many deployment-target warnings during integration:

```bash
IPHONEOS_DEPLOYMENT_TARGET=13.0
```

Set this value to the minimum iOS version supported by your application.

The final command builds a Swift Package for our circuit:

```bash
IPHONEOS_DEPLOYMENT_TARGET=13.0 \
cargo swift --accept-all package \
    -p ios \
    --release
```

After that, the Swift Package will be generated. You can add it to your project as a local package or distribute and integrate it using another method that fits your workflow.

## Final Thoughts

Yeah, if you usually work with only one language, it can be difficult to understand what is happening under the hood. But if you are reading this article, I guess you are already working with some niche technologies. Knowing the basics of Rust will help you grow as an iOS engineer, not just as a SwiftUI specialist. It also gives you more freedom to solve non-trivial tasks faster, without additional discussions or waiting for help from other engineers. 

I didn’t cover all the UniFFI functionality, such as bridging the Tokio runtime for async work or some additional macros that are mostly useful for improving code quality. You can explore these features by yourself in the official documentation. I hope I gave you a basic understanding of what UniFFI is and how you can easily bridge Rust code to Swift.

## References

- [Official UniFFI documentation](https://mozilla.github.io/uniffi-rs/latest/)
- [Complete implementation of the bridged Sudoku circuit](https://github.com/Jonikorjk/how-2-uniffi-swift)
- [cargo-swift repository](https://github.com/antoniusnaumann/cargo-swift)
- [Original Sudoku circuit implementation](https://github.com/tomasdelclaux/ZK-SNARKs)