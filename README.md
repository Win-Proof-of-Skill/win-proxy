# Win Proof of Skill Proxy Server

**Website:** [win-pskill.com](https://win-pskill.com)

A simple proxy server that forwards requests to the Win Proof of Skill (WIN-PSKILL) consensus nodes (validators). The service does nothing special except pick the appropriate node to forward each request to. It aims to spare clients from having to track the nodes themselves.

## Fork Notice

This project is a **fork of [liberdus-proxy](https://github.com/Liberdus/liberdus-proxy)**. We are grateful to the Liberdus team for the foundation this project builds on.

Win Proof of Skill is an independent project and is not affiliated with or endorsed by the Liberdus team. Original copyright and license notices from upstream are retained.

- Upstream project: [Liberdus/liberdus-proxy](https://github.com/Liberdus/liberdus-proxy)
- Upstream network server: [Liberdus/server](https://github.com/Liberdus/server)

## Run with the WIN wallet and server on the same machine

The executable is `win-proxy` and the Rust crate is `win_proxy`. Use the Rust
1.86.0 toolchain specified in `rust-toolchain`; Linux builds also need a C/C++
compiler, CMake, pkg-config, and OpenSSL development headers.

```bash
git clone https://github.com/Win-Proof-of-Skill/win-proxy.git
cd win-proxy
cargo build --release --locked
./target/release/win-proxy
```

Start the WIN server first. The proxy waits for the archiver to report active
validators before opening its HTTP and WebSocket listeners. Run the proxy from
the repository root, or set `WIN_PROXY_CONFIG` to an absolute config-file path.

The checked-in configuration uses services on the **same server**, including when
that server's public address is `162.35.97.118`:

| Service | Address |
| --- | --- |
| WIN archiver | `127.0.0.1:4000` |
| WIN validators | Discovered from the archiver (currently ports 9001–9003) |
| Monitor | `127.0.0.1:3000` |
| Collector API / events | `127.0.0.1:6101` / `127.0.0.1:4444` |
| Notifier | `127.0.0.1:4701` |
| Wallet HTTP gateway | Port `3030` on all interfaces |
| Wallet WebSocket gateway | Port `3031` on all interfaces |

Collector, notifier, and bridge observers are separate services; this repository
does not start them. Basic validator requests do not require those services.
`observer_urls` is empty until WIN observers are deployed and explicitly configured.
Bridge requests return 503 while no observers are configured.

The wallet uses the proxy's client-facing hostname, for example
`http://162.35.97.118:3030` and `ws://162.35.97.118:3031` when accessing that server
from another computer. Its signing `netid` must match the selected WIN server.
A wallet served over HTTPS needs HTTPS/WSS gateway URLs: terminate TLS at your
reverse proxy or enable `tls` with your certificate and key files. The proxy does
not change network IDs, signed transaction payloads, token amounts, or addresses.

## Configuration

Edit `src/config.json` and `src/archiver_seed.json`. The seed's public key must
match your WIN archiver. The checked-in public key matches the existing WIN
server; a freshly generated local network has its own key. The `crypto_seed` is
the Shardus hash key used by WIN, **not** a wallet seed or application network ID.
Never put a wallet private key or archiver secret key in proxy configuration.

Paths inside a config file are resolved relative to that file. For example,
`src/config.json` uses `"archiver_seed_path": "archiver_seed.json"`.
Existing configs using `./src/archiver_seed.json` should change that value when
adopting the new config-file-relative paths. Absolute paths are also supported.

To maintain a separate configuration without editing the defaults:

```bash
mkdir -p config
cp src/config.json config/win.local.json
cp src/archiver_seed.json config/archiver_seed.local.json
```

Set `archiver_seed_path` in `config/win.local.json` to
`archiver_seed.local.json`, update the seed's public key and service addresses as
needed, then run:

```bash
WIN_PROXY_CONFIG=config/win.local.json cargo run --locked
```

For the sibling WIN server's isolated `npm run local:start` network, use only
`archiver.publicKey` from `server/instances-local/local-network.json` in the seed
file. Keep the seed address `127.0.0.1:4000`; do not copy its secret keys. Use that
manifest's `networkId` in the local wallet. No proxy profile starts, resets, or
changes the WIN ledger.

Keep `standalone_network.enabled` **false** when the proxy and WIN server run on
the same machine. Enable it only for a development proxy on a different machine
when the archiver advertises loopback addresses; set `replacement_ip` to that
machine's reachable address. This rewrites discovered archiver and validator IPs.

## API compatibility and checks

WIN routes are forwarded to validators, including `/network/parameters`,
`/account/:id`, `/profiles`, `/contracts`, `/skills`, `/endorsements`, `/scores`,
`/dao/*`, `/transaction/:id`, `/bridge/status`, and `POST /inject`. Existing
WebSocket subscriptions use the next port after `http_port`.

After the WIN network and proxy are running, check a read-only endpoint:

```bash
curl --fail http://127.0.0.1:3030/network/parameters
```

Run the automated checks:

```bash
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets
```

## Contributing

Issues and pull requests are welcome. If a fix or feature is relevant to the shared codebase, please consider contributing it upstream to [Liberdus/liberdus-proxy](https://github.com/Liberdus/liberdus-proxy) as well.

## Links

- Website: [win-pskill.com](https://win-pskill.com)
- Upstream: [github.com/Liberdus/liberdus-proxy](https://github.com/Liberdus/liberdus-proxy)

## License

This project inherits the license of the upstream Liberdus repository. See the `LICENSE` file for details.
