# PGP Tool with Real Post-Quantum Cryptography

A modern web application for PGP operations using **real post-quantum cryptography** via the rpgp Rust library compiled to WebAssembly. Features Dilithium5 digital signatures and Kyber1024 key encapsulation.

## 🚀 Features

- **Real Post-Quantum Cryptography**: Dilithium5 (signatures) and Kyber1024 (encryption)
- **Key Management**: Generate, store, and manage PGP keys with proper passphrase protection
- **Encryption/Decryption**: Secure message encryption with post-quantum algorithms
- **Digital Signatures**: Create and verify detached and clear-text signatures
- **Browser Storage**: Persistent key storage with encryption
- **Dark Mode**: Modern UI with light/dark theme support
- **Crypto Helper**: AI-powered explanations of cryptographic terms

## ⚠️ Important Security Notice

This application implements the **IETF draft specification** for post-quantum cryptography in OpenPGP, which is **NOT finalized** and should **NOT be used in production** environments. This implementation is for **testing, development, and research purposes only**.

## 🛠️ Setup and Installation

### Prerequisites

- Node.js (v16 or later)
- Rust and Cargo (install from [rustup.rs](https://rustup.rs/))
- Git

### Installation Steps

1. **Clone and setup the main application**:
   ```bash
   git clone <your-repo>
   cd <your-project>
   npm install
   ```

2. **Set up the WebAssembly build environment**:
   ```bash
   # Create the Rust WASM project directory
   mkdir rpgp-wasm
   cd rpgp-wasm
   
   # Initialize Rust library project
   cargo init --lib
   ```

3. **Copy the required Rust files**:
   - Copy `Cargo.toml` to `rpgp-wasm/Cargo.toml`
   - Copy `src/lib.rs` to `rpgp-wasm/src/lib.rs`
   - Copy `build-wasm.sh` to `rpgp-wasm/build-wasm.sh`
   - Make the build script executable: `chmod +x build-wasm.sh`

4. **Build the WebAssembly module**:
   ```bash
   cd rpgp-wasm
   ./build-wasm.sh ../public/pkg/
   ```

5. **Configure environment variables**:
   ```bash
   # Create .env.local file in your project root
   echo "GEMINI_API_KEY=your_gemini_api_key_here" > .env.local
   ```

6. **Start the development server**:
   ```bash
   cd .. # Back to main project directory
   npm run dev
   ```

## 📁 Project Structure

```
your-project/
├── rpgp-wasm/                 # Rust WebAssembly project
│   ├── src/lib.rs            # WASM bindings for rpgp
│   ├── Cargo.toml            # Rust dependencies
│   ├── build-wasm.sh         # Build script
│   └── pkg/                  # Generated WASM output
├── public/
│   └── pkg/                  # WASM files for web app
├── services/
│   ├── rpgpService.ts        # Real rpgp integration
│   ├── rpgpMockService.ts    # Compatibility layer
│   ├── storageService.ts     # Browser key storage
│   └── geminiService.ts      # AI crypto explanations
├── components/
│   ├── WasmStatus.tsx        # WebAssembly status indicator
│   └── ... (existing UI components)
└── ... (other project files)
```

## 🔧 Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build the application (includes WASM build)
- `npm run build:wasm` - Build only the WebAssembly module
- `npm run setup` - Initial setup (build WASM and copy files)
- `npm run clean` - Clean all build artifacts

## 🔐 Cryptographic Algorithms

### Primary Key (Signing)
- **Algorithm**: Dilithium5
- **Purpose**: Digital signatures and key certification
- **Security Level**: NIST Level 5 (highest post-quantum security)

### Subkey (Encryption)
- **Algorithm**: Kyber1024
- **Purpose**: Key encapsulation for message encryption
- **Security Level**: NIST Level 5 equivalent

### Symmetric Encryption
- **Algorithm**: AES-256
- **Purpose**: Message content encryption
- **Key Derivation**: Via Kyber1024 KEM

## 🗄️ Key Storage

Keys are securely stored in the browser using:
- **localStorage** for encrypted key material
- **PBKDF2** for passphrase-based key derivation (100,000 iterations)
- **AES-GCM** for private key encryption
- **Random salts** for each stored key

## 🎯 Usage Guide

### Generating Keys
1. Navigate to "Key Management"
2. Enter User ID in format: `Name <email@example.com>`
3. Provide a strong passphrase (minimum 8 characters)
4. Click "Generate Key" and wait for post-quantum key generation

### Encrypting Messages
1. Go to "Encrypt" tab
2. Enter your message
3. Select recipient's public key
4. Click "Encrypt Message"

### Decrypting Messages
1. Go to "Decrypt" tab
2. Paste the encrypted message
3. Select your private key and enter passphrase
4. Click "Decrypt Message"

### Signing Messages
1. Go to "Sign" tab
2. Enter message to sign
3. Select your signing key and enter passphrase
4. Choose "Sign (Clear-Signed)" or "Create Detached Signature"

### Verifying Signatures
1. Go to "Verify" tab
2. Enter original message and signature
3. Select signer's public key
4. Click "Verify Signature"

## 🔍 Crypto Helper

The application includes an AI-powered crypto term explainer using Google's Gemini API. Enter any cryptographic term to get detailed explanations with sources.

## 🐛 Troubleshooting

### WASM Module Fails to Load
- Ensure Rust and wasm-pack are installed
- Check that `public/pkg/` contains the generated WASM files
- Verify your web server serves `.wasm` files correctly

### Key Generation is Slow
- Post-quantum cryptography is computationally intensive
- First-time key generation may take 5-10 seconds
- This is normal behavior for Dilithium5/Kyber1024

### Storage Issues
- Check browser storage quotas
- Clear localStorage if corrupted: `localStorage.clear()`
- Ensure cookies/storage are enabled

### Build Errors
- Verify rpgp repository is accessible
- Check that `draft-pqc` feature is available
- Update Rust toolchain: `rustup update`

## 🔬 Technical Details

### WebAssembly Integration
- Rust code compiled to WASM for browser execution
- JavaScript bindings generated by wasm-bindgen
- Asynchronous API for non-blocking crypto operations

### Post-Quantum Algorithms
- **Dilithium5**: Lattice-based digital signature scheme
- **Kyber1024**: Lattice-based key encapsulation mechanism
- Both algorithms are NIST post-quantum cryptography standards

### Browser Compatibility
- Modern browsers with WebAssembly support
- Chrome 57+, Firefox 52+, Safari 11+, Edge 16+

## 📄 License

This project implements the IETF draft specification for post-quantum cryptography. Please review the rpgp library license and ensure compliance with your use case.

## 🤝 Contributing

This is a demonstration project for post-quantum PGP operations. Contributions are welcome for:
- Performance optimizations
- Additional algorithm support
- UI/UX improvements
- Security enhancements

## 📞 Support

For issues related to:
- **rpgp library**: Check the [rpgp repository](https://github.com/theapplegates/rpgp)
- **Post-quantum crypto**: Refer to NIST PQC documentation
- **WebAssembly**: See wasm-bindgen documentation

---

**Note**: This application preserves the original UI design while adding real cryptographic functionality. The interface remains unchanged to maintain the intended user experience.