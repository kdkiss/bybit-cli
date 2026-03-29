---
name: verify-setup
version: 1.0.0
description: "Confirm that credentials and connectivity are working correctly."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Verify Setup

Confirm that credentials and connectivity are working correctly.

## Check connectivity (no credentials needed)

```bash
# Should return server time
bybit market server-time
```

## Verify credentials

```bash
# Calls /v5/account/info with your configured credentials
bybit auth test
```

A successful response shows your account UID. A failure shows an auth error.

## Interactive setup

```bash
# Configure API key, secret, category, and output format
bybit setup
```

## Manual credential check

```bash
# Set via environment
export BYBIT_API_KEY=your_key
export BYBIT_API_SECRET=your_secret
bybit auth test

# Or via flags
bybit --api-key your_key --api-secret your_secret auth test
```

## Config file location

```bash
# Unix/macOS
cat ~/.config/bybit/config.toml

# Windows
type %APPDATA%\bybit\config.toml
```

## Testnet

```bash
# Testnet credentials are different from mainnet
bybit --testnet auth test

# Or set testnet in config
bybit setup  # select testnet when prompted
```

## Common issues

| Symptom | Fix |
|---------|-----|
| `auth: API key is invalid` | Check key/secret are correct and not expired |
| `auth: API key does not exist` | Key may have been deleted; create a new one |
| `auth: IP not allowed` | Add your IP to the API key's allowed IP list |
| `network: connection refused` | Check internet connectivity |
| Timestamp errors | Sync your system clock (NTP) |
