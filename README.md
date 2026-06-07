# iwdtui

A minimal TUI (Terminal User Interface) for managing Wi-Fi connections via the [iwd](https://iwd.wiki.kernel.org/) backend.

## Features

- List available wireless interfaces
- Scan for nearby networks
- Browse and connect to Wi-Fi networks interactively
- Password prompt for protected networks
- Keyboard-driven navigation

## Dependencies

- [`iwd`](https://archlinux.org/packages/extra/x86_64/iwd/) — IWD wireless daemon (must be running)
- `coreutils`

## Installation

### AUR (recommended)

```bash
paru -S iwdtui
# or
yay -S iwdtui
```

### Manual

```bash
git clone https://github.com/MatiasGabrielAraoz/IwdTUI
cd IwdTUI
cargo build --release
sudo install -Dm755 target/release/iwdtui /usr/bin/iwdtui
```

## Usage

```bash
iwdtui
```

On launch, iwdtui will detect your wireless interfaces. Select one and use the main menu to manage your connection.

### Keybinds

| Key           | Action           |
| ------------- | ---------------- |
| `k ↑` / `j ↓` | Navigate options |
| `Enter` / `→` | Select / confirm |
| `q` / `←`     | Go back / quit   |

### Main menu options

| Option         | Description                          |
|----------------|--------------------------------------|
| `scan`         | Scan for nearby networks             |
| `get-networks` | Refresh and display available networks |
| `connect`      | Open the network selection menu      |

## Notes

- iwdtui requires `iwd` to be active (`systemctl start iwd`).
- Not compatible with Windows.
- If connecting to a protected network fails silently, iwdtui will prompt for the passphrase.

## License

MIT — see [LICENSE](LICENSE).


