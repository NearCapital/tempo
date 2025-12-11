use alloy::{
    genesis::GenesisAccount,
    primitives::{B256, Bytes, U256},
    signers::local::PrivateKeySigner,
};
use clap::Parser;
use eyre::{Context, Result};
use indicatif::{ProgressBar, ProgressIterator};
use rand::{Rng, RngCore, SeedableRng, rngs::StdRng};
use rand_distr::{Distribution, Pareto};
use rayon::prelude::*;
use reth_db_common::init::GenesisAccountWithAddress;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    io::Write,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
};

#[derive(Parser, Debug, Clone)]
#[command(
    version,
    about = "Generate random blockchain state dump compatible with reth init-state",
    long_about = None
)]
struct Args {
    /// Output JSONL file path
    #[arg(short, long)]
    output: PathBuf,

    /// Total number of accounts to generate
    #[arg(long, default_value_t = 1000)]
    accounts: usize,

    /// Percentage of accounts that have storage slots (0.0 to 1.0)
    #[arg(long, default_value_t = 0.1)]
    accounts_with_storage: f64,

    /// Percentage of accounts with contract bytecode (0.0 to 1.0)
    #[arg(long, default_value_t = 0.1)]
    accounts_with_code: f64,

    /// Minimum storage slots per account
    #[arg(long, default_value_t = 1)]
    storage_min: usize,

    /// Maximum storage slots per account
    #[arg(long, default_value_t = 1000)]
    storage_max: usize,

    /// Power-law exponent for storage distribution (higher = more accounts with few slots)
    #[arg(long, default_value_t = 2.0)]
    storage_alpha: f64,

    /// Minimum bytecode size in bytes
    #[arg(long, default_value_t = 100)]
    code_size_min: usize,

    /// Maximum bytecode size in bytes
    #[arg(long, default_value_t = 10000)]
    code_size_max: usize,

    /// Random seed for reproducibility
    #[arg(long)]
    seed: Option<u64>,
}

#[derive(Serialize)]
struct StateRoot {
    root: B256,
}

struct StateGenerator {
    rng: StdRng,
    rng_08: rand_08::rngs::StdRng,
    config: Args,
}

impl StateGenerator {
    fn new(config: Args) -> Self {
        let rng = config
            .seed
            .map(StdRng::seed_from_u64)
            .unwrap_or_else(StdRng::from_os_rng);
        let rng_08 = config
            .seed
            .map(rand_08::SeedableRng::seed_from_u64)
            .unwrap_or_else(rand_08::SeedableRng::from_entropy);
        Self {
            rng,
            rng_08,
            config,
        }
    }

    fn generate_account(&mut self, has_storage: bool, has_code: bool) -> GenesisAccountWithAddress {
        let signer = PrivateKeySigner::random_with(&mut self.rng_08);
        let address = signer.address();
        let balance = U256::random_with(&mut self.rng);
        let nonce = self.rng.random();
        let code = has_code.then(|| self.generate_code());
        let storage = has_storage.then(|| self.generate_storage());

        GenesisAccountWithAddress {
            address,
            genesis_account: GenesisAccount {
                balance,
                nonce: Some(nonce),
                code,
                storage,
                private_key: Some(signer.to_bytes()),
            },
        }
    }

    fn generate_code(&mut self) -> Bytes {
        let size = self
            .rng
            .random_range(self.config.code_size_min..=self.config.code_size_max);
        let mut code = vec![0u8; size];
        self.rng.fill_bytes(&mut code);
        Bytes::from(code)
    }

    fn generate_storage(&mut self) -> BTreeMap<B256, B256> {
        let size = self.generate_storage_size();
        let mut storage = BTreeMap::new();
        for _ in 0..size {
            let key = B256::random_with(&mut self.rng);
            let value = B256::random_with(&mut self.rng);
            storage.insert(key, value);
        }
        storage
    }

    fn generate_storage_size(&mut self) -> usize {
        // Pareto distribution for power-law behavior
        let pareto = Pareto::new(self.config.storage_min as f64, self.config.storage_alpha)
            .expect("Invalid Pareto parameters");
        let size = pareto.sample(&mut self.rng);
        // Clamp to max
        size.min(self.config.storage_max as f64).ceil() as usize
    }
}

fn estimate_size(args: &Args) -> u64 {
    // Base account size (JSONL format):
    // - address: ~44 bytes ("0x" + 40 hex chars)
    // - balance: ~68 bytes (field name + "0x" + up to 64 hex chars)
    // - nonce: ~20 bytes (field name + number)
    // - private_key: ~70 bytes (field name + "0x" + 64 hex chars)
    // - code: ~10 bytes (field name + "0x") when empty
    // - storage: ~15 bytes (field name + {}) when empty
    // - JSON overhead: ~20 bytes (braces, commas, quotes)
    const BASE_ACCOUNT_SIZE: u64 = 250;

    let mut total_size = 0u64;

    // State root line
    total_size += 80; // {"root":"0x00...00"}\n

    // Account base sizes
    total_size += BASE_ACCOUNT_SIZE * args.accounts as u64;

    // Code size for accounts with code
    let accounts_with_code_count = (args.accounts as f64 * args.accounts_with_code) as u64;
    let avg_code_size = (args.code_size_min + args.code_size_max) / 2;
    // In JSON: "0x" + hex encoding (2 chars per byte) + field overhead
    let code_json_size = 2 + (avg_code_size * 2) + 10;
    total_size += (code_json_size as u64) * accounts_with_code_count;

    // Storage size for accounts with storage
    let accounts_with_storage_count = (args.accounts as f64 * args.accounts_with_storage) as u64;
    // Estimate average storage slots using Pareto distribution
    let avg_storage_slots = if args.storage_alpha > 1.0 {
        // Expected value of truncated Pareto (approximation)
        let expected = args.storage_alpha * (args.storage_min as f64) / (args.storage_alpha - 1.0);
        expected.min(args.storage_max as f64) as u64
    } else {
        // For alpha <= 1, use midpoint
        ((args.storage_min + args.storage_max) / 2) as u64
    };

    // Each storage entry in JSON: "0xKEY":"0xVALUE", (~140 bytes)
    const STORAGE_ENTRY_SIZE: u64 = 140;
    let total_storage_entries = avg_storage_slots * accounts_with_storage_count;
    total_size += STORAGE_ENTRY_SIZE * total_storage_entries;

    total_size
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validation
    if args.accounts_with_storage < 0.0 || args.accounts_with_storage > 1.0 {
        return Err(eyre::eyre!(
            "accounts-with-storage ({}) must be between 0.0 and 1.0",
            args.accounts_with_storage
        ));
    }
    if args.accounts_with_code < 0.0 || args.accounts_with_code > 1.0 {
        return Err(eyre::eyre!(
            "accounts-with-code ({}) must be between 0.0 and 1.0",
            args.accounts_with_code
        ));
    }
    if args.storage_min > args.storage_max {
        return Err(eyre::eyre!(
            "storage-min ({}) cannot exceed storage-max ({})",
            args.storage_min,
            args.storage_max
        ));
    }
    if args.code_size_min > args.code_size_max {
        return Err(eyre::eyre!(
            "code-size-min ({}) cannot exceed code-size-max ({})",
            args.code_size_min,
            args.code_size_max
        ));
    }

    let accounts_with_storage_count = (args.accounts as f64 * args.accounts_with_storage) as usize;
    let accounts_with_code_count = (args.accounts as f64 * args.accounts_with_code) as usize;

    println!("Generating state dump with {} accounts", args.accounts);
    println!(
        "  - {} accounts with storage ({:.1}%)",
        accounts_with_storage_count,
        args.accounts_with_storage * 100.0
    );
    println!(
        "  - {} accounts with code ({:.1}%)",
        accounts_with_code_count,
        args.accounts_with_code * 100.0
    );

    // Estimate total size
    let estimated_size = estimate_size(&args);
    println!("\nEstimated output size: {}", format_size(estimated_size));

    // Create output file
    let output_path = args.output.clone();
    let mut output = std::fs::File::create(&output_path).context("Failed to create output file")?;

    // Write dummy state root as first line
    let state_root = StateRoot { root: B256::ZERO };
    writeln!(output, "{}", serde_json::to_string(&state_root)?)?;

    let (tx, rx) = mpsc::channel();
    let total_accounts = args.accounts;
    let seed_inc = Arc::new(AtomicU64::new(0));
    let handle = std::thread::spawn(move || {
        (0..total_accounts).into_par_iter().try_for_each_init(
            || {
                let mut thread_args = args.clone();
                if let Some(seed) = thread_args.seed.as_mut() {
                    *seed += seed_inc.fetch_add(1, Ordering::Relaxed);
                }
                (StateGenerator::new(thread_args), tx.clone())
            },
            |(generator, tx), _| {
                let has_storage = generator.rng.random_bool(args.accounts_with_storage);
                let has_code = generator.rng.random_bool(args.accounts_with_code);

                // Generate account using the method
                let account = generator.generate_account(has_storage, has_code);

                tx.send(account)
                    .map_err(|_| eyre::eyre!("Failed to send account to writer thread"))?;

                eyre::Ok(())
            },
        )
    });

    let progress_bar = ProgressBar::new(total_accounts as u64);
    let progress_bar_hidden = progress_bar.is_hidden();
    for (i, account) in rx.iter().enumerate().progress_with(progress_bar) {
        writeln!(output, "{}", serde_json::to_string(&account)?)?;

        if progress_bar_hidden && i % 1000 == 0 {
            println!("Written {}/{} accounts", i, total_accounts);
        }
    }

    handle.join().expect("failed to join generation thread")?;

    println!(
        "\nState dump written to {} ({} accounts)",
        output_path.display(),
        total_accounts
    );
    Ok(())
}
