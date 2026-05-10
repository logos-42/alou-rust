# Polymarket CLI

A command-line interface for trading and analyzing Polymarket prediction markets.

## Features

- **Market Analysis**: Get detailed analysis of markets including volume, price trends, liquidity, and sentiment
- **Trading**: Place buy/sell orders directly from the CLI
- **Wallet Management**: Connect and manage your Web3 wallet
- **Automated Bots**: Run market making, arbitrage, and trend following strategies
- **Monitoring**: Monitor markets and positions in real-time

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd polymarket-cli

# Build the project
cargo build --release

# Copy example environment file
cp .env.example .env

# Edit .env with your configuration
# Add your private key or mnemonic for trading
```

## Configuration

1. Copy `.env.example` to `.env`
2. Fill in your configuration:
   - `RPC_URL`: Ethereum/Polygon RPC endpoint
   - `PRIVATE_KEY` or `MNEMONIC`: Your wallet credentials (for trading)
   - Contract addresses and other settings

**Security Warning**: Never commit your private key or mnemonic to version control!

## Usage

### List Markets
```bash
polymarket-cli list --active --limit 10
```

### Show Market Details
```bash
polymarket-cli market <market-id>
```

### Analyze Market
```bash
polymarket-cli analyze <market-id> --timeframe 24h
```

### Place a Trade
```bash
polymarket-cli trade <market-id> <outcome> <amount> --price <price> --side <buy/sell>
```

### Wallet Management
```bash
# Show wallet balance
polymarket-cli wallet balance

# Connect wallet
polymarket-cli wallet connect --private-key <key>
# or
polymarket-cli wallet connect --mnemonic "<phrase>"

# Show positions
polymarket-cli wallet positions

# Show transaction history
polymarket-cli wallet history --limit 20
```

### Automated Trading Bots
```bash
# Market making bot
polymarket-cli bot market-maker <market-id> --spread 0.01 --size 100

# Arbitrage bot
polymarket-cli bot arbitrage <market-id1> <market-id2> --threshold 0.01

# Trend following bot
polymarket-cli bot trend <market-id> --stop-loss 0.05 --take-profit 0.10
```

### Monitor Markets
```bash
polymarket-cli monitor --watch --interval 30
```

## Trading Strategies

### 1. Market Making
- Provides liquidity by placing both buy and sell orders
- Earns the spread between bid and ask prices
- Requires capital for both sides of the trade

### 2. Arbitrage
- Exploits price differences between related markets
- Low-risk profit opportunity
- Requires fast execution

### 3. Trend Following
- Identifies and follows market trends
- Uses stop-loss and take-profit orders
- Capitalizes on momentum

## Risk Management

1. **Start Small**: Begin with small amounts to test strategies
2. **Use Stop-Loss**: Always set stop-loss orders to limit losses
3. **Diversify**: Don't put all capital into one market
4. **Monitor Gas Fees**: Be aware of transaction costs on Polygon
5. **Stay Informed**: Keep up with market news and events

## Development

### Building
```bash
cargo build
```

### Testing
```bash
cargo test
```

### Running with Verbose Output
```bash
polymarket-cli --verbose <command>
```

## Security Considerations

- **Private Keys**: Store securely, never share or commit to version control
- **API Keys**: Use environment variables for sensitive data
- **Smart Contracts**: Verify contract addresses before interacting
- **Gas Fees**: Monitor and adjust gas prices for cost-effective transactions
- **Network**: Use testnets for development and testing

## Disclaimer

This software is provided for educational and research purposes only. Trading prediction markets involves significant risk. The authors are not responsible for any financial losses incurred through the use of this software. Always do your own research and understand the risks before trading.

## License

MIT