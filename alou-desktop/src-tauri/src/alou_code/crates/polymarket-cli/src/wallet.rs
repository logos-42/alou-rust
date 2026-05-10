use anyhow::{Context, Result};
use colored::*;
use ethers::prelude::*;
use ethers::signers::coins_bip39::English;
use std::sync::Arc;

const USDC_POLYGON: &str = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359";
const WEI_PER_MATIC: f64 = 1e18;
const WEI_PER_USDC: f64 = 1e6;

pub struct WalletManager {
    client: Arc<Provider<Http>>,
    wallet: Option<LocalWallet>,
}

impl WalletManager {
    pub fn new(provider_url: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(provider_url).context("Invalid RPC URL")?;
        Ok(Self {
            client: Arc::new(provider),
            wallet: None,
        })
    }

    pub fn connect_with_private_key(&mut self, private_key: &str) -> Result<()> {
        let wallet = private_key
            .parse::<LocalWallet>()
            .context("Invalid private key")?;
        self.wallet = Some(wallet);
        Ok(())
    }

    pub fn connect_with_mnemonic(&mut self, mnemonic: &str, index: u32) -> Result<()> {
        let wallet = MnemonicBuilder::<English>::default()
            .phrase(mnemonic)
            .index(index)?
            .build()?;
        self.wallet = Some(wallet);
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.wallet.is_some()
    }

    pub fn address(&self) -> Option<Address> {
        self.wallet.as_ref().map(|w| w.address())
    }

    pub async fn get_matic_balance(&self) -> Result<f64> {
        let addr = self.require_wallet()?;
        let balance = self.client.get_balance(addr, None).await?;
        let wei_f64 = balance.as_u128() as f64;
        let matic = wei_f64 / WEI_PER_MATIC;
        Ok(matic)
    }

    pub async fn get_usdc_balance(&self) -> Result<f64> {
        let addr = self.require_wallet()?;
        let usdc_addr: Address = USDC_POLYGON.parse()?;
        let usdc = Erc20::new(usdc_addr, self.client.clone());

        let balance = usdc.balance_of(addr).call().await?;
        let raw = balance.as_u128() as f64;
        let usdc_val = raw / WEI_PER_USDC;
        Ok(usdc_val)
    }

    fn require_wallet(&self) -> Result<Address> {
        self.wallet
            .as_ref()
            .map(|w| w.address())
            .context("Wallet not connected. Use 'wallet connect' first.")
    }
}

// Minimal ERC-20 ABI for balance_of.
abigen!(
    Erc20,
    r#"[
        function balanceOf(address) view returns (uint256)
    ]"#
);

pub fn print_wallet_status(wallet: &WalletManager) {
    if !wallet.is_connected() {
        println!("{}", "Wallet not connected.".yellow());
        println!("Use: polymarket wallet connect --private-key <KEY>");
        return;
    }
    let addr = wallet.address().unwrap();
    println!("{}", "Wallet Status:".green().bold());
    println!("  Address: {}", format!("{:#x}", addr).dimmed());
}

pub async fn print_balances(wallet: &WalletManager) -> Result<()> {
    print_wallet_status(wallet);
    if !wallet.is_connected() {
        return Ok(());
    }
    let matic = wallet.get_matic_balance().await?;
    let usdc = wallet.get_usdc_balance().await?;
    println!("  MATIC:  {:.4}", matic);
    println!("  USDC:   ${:.2}", usdc);
    Ok(())
}
