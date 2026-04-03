import hashlib
import os
import sys
import time
import requests
from io import BytesIO
from tqdm import tqdm

try:
    from PIL import Image, ImageDraw
    import imagehash
    import matplotlib.pyplot as plt
    from sklearn.metrics import roc_curve, auc
    import numpy as np
except ImportError:
    print("Error: Run 'pip3 install Pillow imagehash requests tqdm matplotlib scikit-learn'")
    sys.exit(1)

NUM_IMAGES = 100
RAW_DIR = "data/raw"
COMP_DIR = "data/compressed"
TAMPER_DIR = "data/tampered"
RESULTS_FILE = "results/benchmark_report.md"
ROC_FILE = "results/roc_curve.png"

for d in [RAW_DIR, COMP_DIR, TAMPER_DIR, "results"]:
    os.makedirs(d, exist_ok=True)

def download_sample_images(count=NUM_IMAGES):
    print(f"[*] Checking dataset ({count} images)...")
    for i in tqdm(range(count), desc="Downloading"):
        img_path = os.path.join(RAW_DIR, f"img_{i:03d}.jpg")
        if not os.path.exists(img_path):
            try:
                response = requests.get("https://picsum.photos/800/600", timeout=10)
                Image.open(BytesIO(response.content)).convert('RGB').save(img_path, "JPEG", quality=100)
            except:
                pass

def simulate_tampering(img):
    """模拟恶意篡改：在图片中央添加一个黑块（模拟抹除关键物体/打水印）"""
    tampered = img.copy()
    draw = ImageDraw.Draw(tampered)
    w, h = tampered.size
    box_size = int(w * 0.15) # 15% 的面积被篡改
    draw.rectangle([w//2 - box_size, h//2 - box_size, w//2 + box_size, h//2 + box_size], fill="black")
    return tampered

def run_benchmark():
    download_sample_images()
    
    noises = {
        "Gateway 95% JPEG": {"quality": 95, "resize": 1.0},
        "Gateway 85% JPEG": {"quality": 85, "resize": 1.0},
        "Aggressive 70% JPEG": {"quality": 70, "resize": 1.0},
    }

    # 用于绘制 ROC 的数据
    y_true =[]      # 1 代表良性操作（应通过），0 代表恶意篡改（应拒绝）
    distances =[]   # 记录对应的 Hamming Distance
    
    print("\n[*] Running Hashing Resilience & Tampering Benchmark...")
    
    for i in tqdm(range(NUM_IMAGES), desc="Benchmarking"):
        raw_path = os.path.join(RAW_DIR, f"img_{i:03d}.jpg")
        if not os.path.exists(raw_path): continue
            
        orig_img = Image.open(raw_path)
        orig_phash = imagehash.phash(orig_img)

        # 1. 模拟良性网络噪声
        for noise_name, params in noises.items():
            comp_path = os.path.join(COMP_DIR, f"img_{i:03d}_{noise_name.replace(' ', '_')}.jpg")
            orig_img.copy().save(comp_path, "JPEG", quality=params["quality"])
            new_phash = imagehash.phash(Image.open(comp_path))
            
            y_true.append(1)
            distances.append(orig_phash - new_phash)
            
        # 2. 模拟恶意篡改 (Deepfake/Photoshop 涂抹)
        tamper_path = os.path.join(TAMPER_DIR, f"img_{i:03d}_tampered.jpg")
        tampered_img = simulate_tampering(orig_img)
        tampered_img.save(tamper_path, "JPEG", quality=95)
        tampered_phash = imagehash.phash(Image.open(tamper_path))
        
        y_true.append(0)
        distances.append(orig_phash - tampered_phash)

    # 绘制 ROC 和分布图
    plot_roc_and_distribution(y_true, distances)

def plot_roc_and_distribution(y_true, distances):
    # 将距离转换为“相似度得分”用于画 ROC，距离越小，得分越高 (64 - distance)
    scores = [64 - d for d in distances]
    fpr, tpr, thresholds = roc_curve(y_true, scores)
    roc_auc = auc(fpr, tpr)

    plt.figure(figsize=(12, 5))

    # 子图1：ROC 曲线
    plt.subplot(1, 2, 1)
    plt.plot(fpr, tpr, color='darkorange', lw=2, label=f'ROC curve (AUC = {roc_auc:.4f})')
    plt.plot([0, 1], [0, 1], color='navy', lw=2, linestyle='--')
    plt.xlim([0.0, 1.0])
    plt.ylim([0.0, 1.05])
    plt.xlabel('False Positive Rate (Accepted Tampered)')
    plt.ylabel('True Positive Rate (Accepted Benign)')
    plt.title('Receiver Operating Characteristic')
    plt.legend(loc="lower right")

    # 子图2：良性 vs 恶意 距离分布直方图
    plt.subplot(1, 2, 2)
    benign_dists =[d for y, d in zip(y_true, distances) if y == 1]
    tamper_dists =[d for y, d in zip(y_true, distances) if y == 0]
    
    plt.hist(benign_dists, bins=range(0, 25), alpha=0.7, label='Benign Compression', color='green')
    plt.hist(tamper_dists, bins=range(0, 25), alpha=0.7, label='Malicious Tampering', color='red')
    plt.axvline(x=5, color='blue', linestyle='dashed', linewidth=2, label='Proposed ZK Threshold (5)')
    
    plt.xlabel('Hamming Distance')
    plt.ylabel('Frequency')
    plt.title('Hamming Distance Distribution')
    plt.legend(loc='upper right')

    plt.tight_layout()
    plt.savefig(ROC_FILE)
    print(f"\n[*] Visual plots saved to: {ROC_FILE}")

if __name__ == "__main__":
    run_benchmark()