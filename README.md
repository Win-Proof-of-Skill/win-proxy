# Win Proof of Skill Proxy Server

**Website:** [win-pskill.com](https://win-pskill.com)

A simple proxy server that forwards requests to the Win Proof of Skill (WIN-PSKILL) consensus nodes (validators). The service does nothing special except pick the appropriate node to forward each request to. It aims to spare clients from having to track the nodes themselves.

## Fork Notice

This project is a **fork of [liberdus-proxy](https://github.com/Liberdus/liberdus-proxy)**. We are grateful to the Liberdus team for the foundation this project builds on.

Win Proof of Skill is an independent project and is not affiliated with or endorsed by the Liberdus team. Original copyright and license notices from upstream are retained.

- Upstream project: [Liberdus/liberdus-proxy](https://github.com/Liberdus/liberdus-proxy)
- Upstream network server: [Liberdus/server](https://github.com/Liberdus/server)

## Prerequisites

Install Cargo (Rust toolchain) if you don't have it:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Setup

Clone the repository:

```bash
git clone <YOUR-WIN-PSKILL-PROXY-REPO-URL> win-pskill-proxy
cd win-pskill-proxy
```

Provide the seed archiver in `./src/seed_archiver.json`.

## Standalone network

Use this mode when the entire network runs on a remote machine that is different from the proxy server. In that case the archiver returns a list of validators with loopback IPs (because they share a machine), and the loopback addresses break the proxy. Standalone network mode replaces them with a reachable IP.

Configure it in `src/config.json`:

```json
{
    "standalone_network": {
        "enabled": true,
        "replacement_ip": "[IP of the machine that hosts the network you want to connect to]"
    }
}
```

Also configure the seed archiver in `./src/seed_archiver.json` with the same IP and credentials.

> **Note:** Standalone network mode must never be used in production.

## Run the server

Make sure your current working directory is the root of the repository, then run:

```bash
cargo run
```

## Contributing

Issues and pull requests are welcome. If a fix or feature is relevant to the shared codebase, please consider contributing it upstream to [Liberdus/liberdus-proxy](https://github.com/Liberdus/liberdus-proxy) as well.

## Links

- Website: [win-pskill.com](https://win-pskill.com)
- Upstream: [github.com/Liberdus/liberdus-proxy](https://github.com/Liberdus/liberdus-proxy)

## License

This project inherits the license of the upstream Liberdus repository. See the `LICENSE` file for details.
