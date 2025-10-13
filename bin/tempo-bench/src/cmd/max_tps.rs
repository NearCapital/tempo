use crate::config::BenchConfig;
use alloy::{
    network::TxSignerSync,
    primitives::{Address, TxKind, U256},
    providers::{Provider, ProviderBuilder},
    sol,
    sol_types::SolCall,
    transports::http::reqwest::Url,
};
use alloy_consensus::{SignableTransaction, TxLegacy};
use alloy_signer_local::{MnemonicBuilder, PrivateKeySigner, coins_bip39::English};
use clap::Parser;
use core_affinity::CoreId;
use eyre::{Context, ensure};
use governor::{Quota, RateLimiter};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rlimit::Resource;
use simple_tqdm::ParTqdm;
use std::{
    num::NonZeroU32,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};
use tempo_chainspec::spec::TEMPO_BASE_FEE;
use tokio::time::timeout;

sol! {
    interface ERC20 {
        function transfer(address to, uint256 amount) external returns (bool);
    }
}

/// Run maximum TPS throughput benchmarking
#[derive(Parser, Debug)]
pub struct MaxTpsArgs {
    /// Path to TOML configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// Target transactions per second
    #[arg(short, long)]
    tps: Option<u64>,

    /// Test duration in seconds
    #[arg(short, long)]
    duration: Option<u64>,

    /// Number of accounts for pre-generation
    #[arg(short, long)]
    accounts: Option<u64>,

    /// Number of workers to send transactions
    #[arg(short, long)]
    workers: Option<usize>,

    /// Mnemonic for generating accounts
    #[arg(short, long)]
    mnemonic: Option<String>,

    /// Chain ID
    #[arg(long)]
    chain_id: Option<u64>,

    /// Token address used when creating TIP20 transfer calldata
    #[arg(long)]
    token_address: Option<Address>,

    /// Target URLs for network connections
    #[arg(long)]
    target_urls: Option<Vec<String>>,

    /// Total network connections
    #[arg(long)]
    total_connections: Option<u64>,

    /// Disable binding worker threads to specific CPU cores, letting the OS scheduler handle placement.
    #[arg(long)]
    disable_thread_pinning: Option<bool>,

    /// File descriptor limit to set
    #[arg(long)]
    fd_limit: Option<u64>,
}

impl MaxTpsArgs {
    pub async fn run(self) -> eyre::Result<()> {
        // Load config from file if provided
        let config = if let Some(config_path) = &self.config {
            BenchConfig::from_file(config_path)
                .with_context(|| format!("Failed to load config from {}", config_path))?
        } else {
            BenchConfig::default()
        };

        let benchmark_config = config.benchmark.unwrap_or_default();

        // Merge CLI args with config (CLI args take precedence)
        let tps = self
            .tps
            .or(benchmark_config.initial_ratelimit)
            .ok_or_else(|| eyre::eyre!("TPS must be specified via --tps or config file"))?;

        let duration = self.duration.or(benchmark_config.duration).unwrap_or(30);

        let accounts = self.accounts.or(benchmark_config.accounts).unwrap_or(100);

        let workers = self.workers.or(benchmark_config.workers).unwrap_or(10);

        let mnemonic = self
            .mnemonic
            .or(benchmark_config.mnemonic)
            .unwrap_or_else(|| {
                "test test test test test test test test test test test junk".to_string()
            });

        let chain_id = self.chain_id.or(benchmark_config.chain_id).unwrap_or(1337);

        let token_address = self
            .token_address
            .or(benchmark_config.token_address)
            .unwrap_or_else(|| "0x20c0000000000000000000000000000000000000".parse().unwrap());

        let target_urls = self
            .target_urls
            .or(benchmark_config.target_urls)
            .unwrap_or_else(|| vec!["http://localhost:8545".to_string()]);

        let total_connections = self
            .total_connections
            .or(benchmark_config.total_connections)
            .unwrap_or(100);

        let disable_thread_pinning = self
            .disable_thread_pinning
            .or(benchmark_config.disable_thread_pinning)
            .unwrap_or(false);

        let fd_limit = self.fd_limit.or(benchmark_config.fd_limit);

        // Set file descriptor limit if provided
        if let Some(fd_limit) = fd_limit {
            increase_nofile_limit(fd_limit).context("Failed to increase nofile limit")?;
        }

        let target_urls: Vec<Url> = target_urls
            .iter()
            .map(|s| {
                s.parse::<Url>()
                    .wrap_err_with(|| format!("failed to parse `{s}` as URL"))
            })
            .collect::<eyre::Result<Vec<_>>>()
            .wrap_err("failed parsing input target URLs")?;

        // Generate all transactions
        let total_txs = tps * duration;
        let transactions = Arc::new(
            generate_transactions(
                total_txs,
                accounts,
                &mnemonic,
                chain_id,
                token_address,
                &target_urls[0],
            )
            .await
            .context("Failed to generate transactions")?,
        );

        // Create shared transaction counter and monitoring
        let tx_counter = Arc::new(AtomicU64::new(0));

        // Spawn monitoring thread for TPS tracking
        let _monitor_handle = monitor_tps(tx_counter.clone());

        // Spawn workers and send transactions
        send_transactions(
            transactions,
            workers,
            total_connections,
            target_urls,
            tps,
            disable_thread_pinning,
            tx_counter,
        )
        .context("Failed to send transactions")?;

        // Wait for all sender threads to finish
        std::thread::sleep(Duration::from_secs(duration));
        println!("Finished sending transactions");

        Ok(())
    }
}

fn send_transactions(
    transactions: Arc<Vec<Vec<u8>>>,
    num_workers: usize,
    _num_connections: u64,
    target_urls: Vec<Url>,
    tps: u64,
    disable_thread_pinning: bool,
    tx_counter: Arc<AtomicU64>,
) -> eyre::Result<()> {
    // Get available cores
    let core_ids =
        core_affinity::get_core_ids().ok_or_else(|| eyre::eyre!("Failed to get core IDs"))?;
    println!("Detected {} effective cores.", core_ids.len());

    let num_sender_threads = num_workers.min(core_ids.len());
    let chunk_size = transactions.len().div_ceil(num_sender_threads);

    // Create a shared rate limiter for all threads
    let rate_limiter = Arc::new(RateLimiter::direct(Quota::per_second(
        NonZeroU32::new(tps as u32).unwrap(),
    )));

    for thread_id in 0..num_sender_threads {
        if !disable_thread_pinning {
            let core_id = core_ids[thread_id % core_ids.len()];
            pin_thread(core_id);
        }

        // Segment transactions
        let rate_limiter = rate_limiter.clone();
        let transactions = transactions.clone();
        let target_urls = target_urls.to_vec();
        let tx_counter = tx_counter.clone();
        let start = thread_id * chunk_size;
        let end = (start + chunk_size).min(transactions.len());

        // Spawn thread and send transactions over specified duration
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build tokio runtime");

            rt.block_on(async {
                // TODO: Send txs from multiple senders
                // Create multiple connections for this thread
                // let mut providers = Vec::new();
                // for i in 0..num_connections {
                //     println!("{i:?}");
                //     let url = &target_urls[(i as usize) % target_urls.len()];
                //     let provider = ProviderBuilder::new().connect_http(url.clone());
                //     providers.push(provider);
                // }

                let provider = ProviderBuilder::new().connect_http(target_urls[0].clone());
                for tx_bytes in transactions[start..end].iter() {
                    rate_limiter.until_ready().await;

                    match timeout(
                        Duration::from_secs(1),
                        provider.send_raw_transaction(tx_bytes),
                    )
                    .await
                    {
                        Ok(Ok(_)) => {
                            tx_counter.fetch_add(1, Ordering::Relaxed);
                        }
                        Ok(Err(e)) => eprintln!("Failed to send transaction: {}", e),
                        Err(_) => eprintln!("Tx send timed out"),
                    }
                }
            });
        });
    }

    Ok(())
}

async fn generate_transactions(
    total_txs: u64,
    num_accounts: u64,
    mnemonic: &str,
    chain_id: u64,
    token_address: Address,
    rpc_url: &Url,
) -> eyre::Result<Vec<Vec<u8>>> {
    println!("Generating {num_accounts} accounts...");
    let signers: Vec<PrivateKeySigner> = (0..num_accounts as u32)
        .into_par_iter()
        .tqdm()
        .map(|i| -> eyre::Result<PrivateKeySigner> {
            let signer = MnemonicBuilder::<English>::default()
                .phrase(mnemonic)
                .index(i)?
                .build()?;
            Ok(signer)
        })
        .collect::<eyre::Result<Vec<_>>>()?;

    let txs_per_sender = total_txs / num_accounts;
    ensure!(
        txs_per_sender > 0,
        "txs per sender is 0, increase tps or decrease senders"
    );

    // Fetch current nonces for all accounts
    let provider = ProviderBuilder::new().connect_http(rpc_url.clone());
    println!("Fetching nonces for {} accounts...", signers.len());

    let mut params = Vec::new();
    for signer in signers {
        let address = signer.address();
        let current_nonce = provider
            .get_transaction_count(address)
            .await
            .context("Failed to get transaction count")?;

        for i in 0..txs_per_sender {
            params.push((signer.clone(), current_nonce + i));
        }
    }

    let transactions: Vec<Vec<u8>> = params
        .into_par_iter()
        .tqdm()
        .map(|(signer, nonce)| -> eyre::Result<Vec<u8>> {
            let mut tx = TxLegacy {
                chain_id: Some(chain_id),
                nonce,
                gas_price: TEMPO_BASE_FEE as u128,
                gas_limit: 30000,
                to: TxKind::Call(token_address),
                value: U256::ZERO,
                input: ERC20::transferCall {
                    to: Address::random(),
                    amount: U256::ONE,
                }
                .abi_encode()
                .into(),
            };

            let signature = signer
                .sign_transaction_sync(&mut tx)
                .map_err(|e| eyre::eyre!("Failed to sign transaction: {}", e))?;
            let mut payload = Vec::new();
            tx.into_signed(signature).eip2718_encode(&mut payload);
            Ok(payload)
        })
        .collect::<eyre::Result<Vec<_>>>()?;

    println!("Generated {} transactions", transactions.len());
    Ok(transactions)
}

pub fn increase_nofile_limit(min_limit: u64) -> eyre::Result<u64> {
    let (soft, hard) = Resource::NOFILE.get()?;
    println!("[*] At startup, file descriptor limit:      soft = {soft}, hard = {hard}");

    if hard < min_limit {
        panic!(
            "[!] File descriptor hard limit is too low. Please increase it to at least {min_limit}."
        );
    }

    if soft != hard {
        Resource::NOFILE.set(hard, hard)?; // Just max things out to give us plenty of overhead.
        let (soft, hard) = Resource::NOFILE.get()?;
        println!("[+] After increasing file descriptor limit: soft = {soft}, hard = {hard}");
    }

    Ok(soft)
}

/// Pin the current thread to the given core ID if enabled.
/// Panics if the thread fails to pin.
pub fn pin_thread(core_id: CoreId) {
    if !core_affinity::set_for_current(core_id) {
        panic!(
            "[!] Failed to pin thread to core {}. Try disabling thread_pinning in your config.",
            core_id.id
        );
    }
}

fn monitor_tps(tx_counter: Arc<AtomicU64>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut last_count = 0u64;
        loop {
            let current_count = tx_counter.load(Ordering::Relaxed);
            let tps = current_count - last_count;
            last_count = current_count;

            println!("TPS Sent: {tps}, Total Txs Sent: {current_count}");
            thread::sleep(Duration::from_secs(1));
        }
    })
}
