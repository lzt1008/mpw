# tpw
tpw is a lightweight macOS command-line tool for monitoring and displaying real-time power usage statistics

<div align="center">
  <img src="https://github.com/lzt1008/tpw/blob/assets/tpw.png?raw=true" alt="preview" />
</div>

<br />
<br />

## Features

- Real-time power consumption tracking for CPU, GPU, DRAM, ANE and other components.
- Lightweight and fast, with minimal resource usage.
- No root privileges required.

## Installation

### Using Homebrew (Recommended)

You can install `tpw` using Homebrew:

```bash
brew tap lzt1008/tpw
brew install tpw
```

### Using Cargo

You can install `tpw` using Cargo:

```bash
cargo install tpw
```

## Usage

```bash
Usage: tpw [OPTIONS]

Options:
  -i, --interval <INTERVAL>  Interval in milliseconds [default: 1000]
  -h, --help                 Print help
```

## License

This project is licensed under the MIT License.

## See Also

- [vladkens/macmon](github.com/vladkens/macmon) - Access CPU, RAM, GPU, and other system information from the command line.
 