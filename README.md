# i386-emu

Welcome to `i386-emu`, an emulator for the Intel 386 (i386) CPU architecture, written in Rust. This project aims to provide a platform for emulating i386 systems, allowing users to run and test i386 binaries in a controlled environment.

## Getting Started

To use `i386-emu`, you'll need to have Docker and Rust's cargo installed on your machine.

### Prerequisites

- Install [Docker](https://www.docker.com/get-started)
- Install [Rust and Cargo](https://www.rust-lang.org/tools/install)

### Running the Emulator

1. Clone the repository to your local machine:

```bash
git clone https://github.com/yourusername/i386-emu.git
cd i386-emu
```

2. Start the Docker environment:

```bash
docker compose up -d
```

This command will set up the necessary environment and generate a `program.bin` file, which is a binary file that you can use to test the emulator.

3. Run the emulator with the generated binary:

```bash
cargo run program.bin
```

This will start the i386 emulation using the `program.bin` as input.

## Development Status

Please note that `i386-emu` is currently under active development. Features may be added or changed, and stability is not guaranteed
