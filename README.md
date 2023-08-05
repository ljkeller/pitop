# PiTop - Rust Client-Server App for System Utilization Monitoring

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

PiTop is a simple, sleek Rust client-server application designed to monitor system utilization from Windows client and display the processed data on a Raspberry Pi server (I display the data on the 7" raspberry pi display) The app uses various rust libraries, such as [tui](https://docs.rs/tui/latest/tui/), [sysinfo](https://docs.rs/sysinfo/latest/sysinfo/), [clap](https://docs.rs/clap/latest/clap/). The app is built on TCP with a MPSC design on the pi server.
![PiTopDemo](https://github.com/ljkeller/pitop/assets/44109284/f1ecf218-8e23-4af0-b500-77496f0eb8fa)

## Features

- Gather system utilization data on a Windows client (Supports NVIDIA systems)
- Send the collected data to the Raspberry Pi server.
- Display real-time system utilization information on the Raspberry Pi using a terminal-based user interface (`tui`).

## Requirements

- Rust programming language (installation guide: [Rust Lang](https://www.rust-lang.org/learn/get-started))
- Cargo
- Windows Client
- Server (Windows and Linux supported)

## Installation
Clone the repository:

```bash
git clone https://github.com/your-username/PiTop.git
```

## Usage

1. Run the Windows client:

```bash
cargo run --bin win_client
```

2. Run the Raspberry Pi server:

```bash
cargo run --bin pi_server
```
> command line arguments are supported: `cargo run --bin win_client -- --help` for more info

## License

This project is licensed under the terms of the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvements, feel free to open an issue or create a pull request.

## Acknowledgments
Thank you to the creators of the [tui](https://docs.rs/tui/latest/tui/) and [sysinfo](https://docs.rs/sysinfo/latest/sysinfo/) libraries for making this app possible!
