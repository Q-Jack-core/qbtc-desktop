# Q-BTC Desktop

Self-custody **ML-DSA-65** post-quantum vault and local full-node verifier for the Quantum Bitcoin (Q-BTC) network, built with Tauri and Rust.

---

## 💬 Genesis Community

To follow protocol upgrades, share ecosystem tools, and connect with independent miners and builders, join the Genesis community:

👉 **Telegram (QBTC Genesis Members):** [https://t.me/qbtc_genesis](https://t.me/qbtc_genesis)

---

## 📥 Download

Visit the [Releases page](../../releases/latest) to download the latest installer for your operating system:
* **macOS:** Download the `.dmg` file (Universal binary supporting both Intel and Apple Silicon).
* **Windows:** Download the `.exe` or `.msi` installer.

---

## 🍎 macOS Installation & Troubleshooting

Because this open-source build is distributed without an Apple Developer certificate, macOS Gatekeeper may block the initial launch. Use either method below to open it:

### Method 1: System Settings (Recommended)
1. Double-click the `.dmg` file and drag `qbtc-desktop` into your **Applications** folder.
2. Open **Applications** and double-click `qbtc-desktop`. Click **OK** or **Cancel** on the prompt.
3. Open **System Settings** -> **Privacy & Security**.
4. Scroll down to the **Security** section where `"qbtc-desktop" was blocked from use` appears.
5. Click **Open Anyway** and confirm with your Mac password or Touch ID.

### Method 2: Terminal Quick Fix
Run the following command in **Terminal** to clear the quarantine attribute:
```bash
xattr -cr /Applications/qbtc-desktop.app
```
Then open `qbtc-desktop` normally from Applications.

---

## 🪟 Windows Installation & Troubleshooting

1. Download and run `qbtc-desktop_x64-setup.exe`.
2. If Windows Defender SmartScreen displays *"Windows protected your PC"*, click **More info** -> **Run anyway**.
3. On first launch, if Windows Firewall prompts for network permission, click **Allow access** so the embedded full node can sync blocks over the P2P network.

### 🛠 Common Windows Errors

**Error: "VCRUNTIME140.dll or MSVCP140.dll was not found"**
Clean Windows installations may lack the standard C++ runtime libraries required by the Rust node engine.
* **Fix:** Install the Microsoft Visual C++ Redistributable package: [vc_redist.x64.exe (Microsoft Download)](https://aka.ms/vs/17/release/vc_redist.x64.exe)
* Relaunch Q-BTC Desktop after installation (no reboot required).

---

## 🚀 Quick Start & Mining Isolation

1. **Wait for Sync:** Launch Q-BTC Desktop and wait for the bottom-left indicator to turn green (`SYNCHRONIZED`).
2. **Create a Vault:** In **Wallet Overview**, set a wallet alias and encryption password, click **Create Wallet**, and securely back up your 12-word recovery phrase offline.
3. **Mine to Your Address:** Built-in pool mining has been removed to keep private keys isolated from external mining environments. Copy your `qbtc1...` address and point any open-source community pool miner (CPU / CUDA / OpenCL) or local **Q-BTC Core CLI** solo miner (`auto_mine start <address>`) directly to your receiving address.