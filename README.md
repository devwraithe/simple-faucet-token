Step 3: Running the Builds
You can now build and test separately based on the target:

For BPF Build: Run the following command to compile your Solana program without test dependencies:

bash
Copy code
cargo build-bpf --no-default-features
For Native Tests: Use cargo test to run the native tests with the solana-program-test crate:

bash
Copy code
cargo test --features test-bpf
This setup keeps your BPF build clean and free from testing dependencies while allowing you to run tests on the native architecture using solana-program-test.