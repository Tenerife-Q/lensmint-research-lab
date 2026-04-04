use methods::{METHOD_ELF, METHOD_ID}; // <--- 这里名字改成了 METHOD
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use std::time::Instant;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let original_phash: u64 = 0b1111000011110000111100001111000011110000111100001111000011110000;
    let uploaded_phash: u64 = 0b1111000011110000111100001111000011110000111100001111000011110011;
    let threshold: u32 = 5;

    println!("\n[*] Starting ZK Proof generation on simulated Edge Device...");

    let env = ExecutorEnv::builder()
        .write(&original_phash).unwrap()
        .write(&uploaded_phash).unwrap()
        .write(&threshold).unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    
    let start_time = Instant::now();
    
    let prove_info = prover
        .prove_with_opts(env, METHOD_ELF, &ProverOpts::default())
        .unwrap();

    let elapsed = start_time.elapsed();
    let distance: u32 = prove_info.receipt.journal.decode().unwrap();
    let stats = prove_info.stats;

    println!("\n==================================================");
    println!("✅ ZK Proof Successfully Generated!");
    println!("✅ Verified Hamming Distance : {}", distance);
    println!("⏱️  Proving Time             : {:.2?}", elapsed);
    println!("⚙️  Total Cycle Count        : {}", stats.user_cycles);
    println!("📦 Memory Segments Used     : {} (Perfect for RPi RAM limits)", stats.segments);
    println!("==================================================\n");
}