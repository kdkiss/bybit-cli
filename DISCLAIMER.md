# Disclaimer

**bybit-cli is an unofficial, community-built tool. It is not affiliated with, endorsed by, or supported by Bybit.**

## Financial Risk

- Cryptocurrency trading involves substantial risk of loss.
- This tool can place real orders, transfer funds, and withdraw to external addresses.
- **Always test commands with `--validate` (dry-run) or on testnet (`--testnet`) before using with real funds.**
- The authors are not responsible for any financial losses arising from the use of this software.

## No Warranty

This software is provided "as is", without warranty of any kind, express or implied. In no event shall the authors be liable for any claim, damages, or other liability arising from the use of this software.

## Security

- API credentials are stored in `~/.config/bybit/config.toml` with restricted permissions (0600 on Unix).
- Never share your API key or secret with anyone.
- Use IP-restricted API keys where possible.
- Disable withdrawal permissions on your API key if you do not need them.

## Compliance

- You are responsible for complying with all applicable laws and regulations in your jurisdiction regarding cryptocurrency trading.
- Bybit may not be available in all regions. Check [Bybit's Terms of Service](https://www.bybit.com/en-US/legal/terms-service.html) before use.

## Rate Limits

Excessive use of this tool may trigger Bybit rate limits and result in temporary API key suspension. The CLI includes automatic retry with exponential backoff, but it is your responsibility to use the API within Bybit's published limits.
