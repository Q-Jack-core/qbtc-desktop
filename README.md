# Q-BTC Desktop Miner

The Official GUI Miner for the Quantum Bitcoin (Q-BTC) network. A seamless, one-click mining experience built with Tauri and Rust.

---

## 💬 Official Community & 20,000 Blocks Milestone

**The Q-BTC mainnet has successfully crossed the 20,000 block milestone!** 
To coordinate network upgrades, gather miner feedback, and discuss the future roadmap, all active miners and early believers are welcome to join the Genesis Members.

👉 **Official Telegram:** [https://t.me/qbtc_genesis](https://t.me/qbtc_genesis)

---

## 📥 Download

Go to the [Releases page](../../releases/latest) to download the latest version for your operating system:
* **macOS:** Download the `.dmg` file (Universal binary, supports both Intel and Apple Silicon M-series).
* **Windows:** Download the `.exe` setup file.

---

## 🍎 macOS Installation & Troubleshooting

Because this is an open-source application without an Apple Developer certificate, macOS Gatekeeper will block the initial launch. Please follow one of the methods below to open it:

### Method 1: System Settings (Recommended)
1. Double-click the `.dmg` file and drag `qbtc-desktop` into your **Applications** folder.
2. Go to your Applications folder and double-click `qbtc-desktop`. You will see a warning that it cannot be opened. Click **OK** or **Cancel**.
3. Open **System Settings** -> **Privacy & Security**.
4. Scroll down to the "Security" section. You will see a message saying *"qbtc-desktop" was blocked from use because it is not from an identified developer*.
5. Click the **Open Anyway** button next to it.
6. Enter your Mac password or use Touch ID, and the miner will launch.

### Method 2: Terminal Quick Fix (Pro Users)
If you want to bypass the security settings instantly, open your **Terminal** and run this command to remove the quarantine attribute:
```bash
xattr -cr /Applications/qbtc-desktop.app
```
After running this, you can double-click the app to open it normally.

---

## 🪟 Windows Installation & Troubleshooting

1. Download and run the `qbtc-desktop_x64-setup.exe` file[cite: 1].
2. **Note:** Windows Defender SmartScreen may display a blue warning screen saying "Windows protected your PC"[cite: 1]. 
3. Click **More info**, then click **Run anyway** to proceed with the installation[cite: 1].
4. Upon first launch, Windows Security may prompt a Firewall alert. Click **Allow access** to enable the miner to connect to the external Stratum Pool.

### 🛠 Common Windows Errors

**Error: "VCRUNTIME140.dll or MSVCP140.dll was not found"**
This happens on clean Windows operating systems that lack the standard C++ runtime libraries required by the Rust core engine.
* **Fix:** Download and install the official Microsoft Visual C++ Redistributable package.
* **Direct Download Link:** [vc_redist.x64.exe (Official Microsoft Link)](https://aka.ms/vs/17/release/vc_redist.x64.exe)
* After a quick installation, restart the Q-BTC Desktop Miner. (No system reboot is required).

---

## 🚀 Quick Start
1. Launch the Q-BTC Desktop Miner.
2. Enter your Q-BTC Wallet Address (Coinbase Reward Target).
3. Click **START MINING** to connect to the Stratum Pool and begin hashing.

### ⚠️ Important: Receiving Mining Rewards
Before starting the miner, ensure you enter your **real Q-BTC Wallet Address** (e.g., `qbtc1...`) in the "Pool Mining" tab, not an alias or name. 
Mining with a fake name or alias will result in permanently lost rewards. If you don't have a wallet yet, you can create one in the "Wallet Overview" tab.
