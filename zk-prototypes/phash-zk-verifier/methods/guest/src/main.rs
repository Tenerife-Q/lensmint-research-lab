use risc0_zkvm::guest::env;

fn main() {
    let original_phash: u64 = env::read();
    let uploaded_phash: u64 = env::read();
    let threshold: u32 = env::read();

    let distance = (original_phash ^ uploaded_phash).count_ones();

    if distance > threshold {
        panic!("Tamper detected: Hamming distance {} exceeds threshold {}", distance, threshold);
    }

    env::commit(&distance);
}