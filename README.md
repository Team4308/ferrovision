# Ferrovision

Ferrovision is a FRC vision system created in rust for high performance and extreme accuracy.

## Install

### Prerequisites

- A raspberry pi 3 or 4
- A raspberry pi camera v2

1. Clone https://gist.github.com/1473d39a706ba07c012319f27b259b8f.git.
2. Run the script with `./setup_ferrovision.sh --install`

## Configuration

All configuration for ferrovision is done with `.toml` files as that is what rust uses.

The main configuration is file is `vset.toml` located at `/ferrovision/vset.toml`. It contains the pipeline definition as well module specific configuration.
