import { RpgpPublicKey, RpgpKeyPair, GenerateKeyParams, EncryptParams, DecryptParams, SignParams, VerifyParams } from '../types';
import { storageService } from './storageService';

// WebAssembly module interface
interface RPGPWasm {
  generate_key_pair(params: any): Promise<any>;
  get_public_key(keyId: string): Promise<any>;
  get_all_public_keys(): Promise<any>;
  encrypt_message(params: any): Promise<string>;
  decrypt_message(params: any): Promise<string>;
  sign_message(params: any): Promise<string>;
  create_detached_signature(params: any): Promise<string>;
  verify_message(params: any): Promise<any>;
}

class RPGPService {
  private wasm: RPGPWasm | null = null;
  private initPromise: Promise<void> | null = null;

  private async initializeWasm(): Promise<void> {
    if (this.wasm) {
      return;
    }

    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = this.loadWasm();
    return this.initPromise;
  }

  private async loadWasm(): Promise<void> {
    try {
      // Import the WebAssembly module
      // Note: The path should match where you place the built WASM files
      const wasmModule = await import('/pkg/rpgp_wasm.js');
      await wasmModule.default(); // Initialize the WASM module
      
      this.wasm = {
        generate_key_pair: wasmModule.generate_key_pair,
        get_public_key: wasmModule.get_public_key,
        get_all_public_keys: wasmModule.get_all_public_keys,
        encrypt_message: wasmModule.encrypt_message,
        decrypt_message: wasmModule.decrypt_message,
        sign_message: wasmModule.sign_message,
        create_detached_signature: wasmModule.create_detached_signature,
        verify_message: wasmModule.verify_message,
      };

      console.log('✅ RPGP WebAssembly module loaded successfully with post-quantum cryptography support');
    } catch (error) {
      console.error('❌ Failed to load RPGP WebAssembly module:', error);
      throw new Error(
        'Failed to initialize RPGP WebAssembly module. Please ensure the WASM files are built and accessible. ' +
        'Run the build script to generate the WebAssembly module.'
      );
    }
  }

  async generateKeyPair(params: GenerateKeyParams): Promise<RpgpKeyPair> {
    await this.initializeWasm();
    
    if (!this.wasm) {
      throw new Error('RPGP WebAssembly module not initialized');
    }

    if (!params.userId.match(/^[^<]+<[^@]+@[^>]+>$/)) {
      throw new Error('Invalid User ID format. Expected "Name <email@example.com>".');
    }

    if (!params.passphrase || params.passphrase.length < 8) {
      throw new Error('Passphrase is required and must be at least 8 characters long for security.');
    }

    try {
      const result = await this.wasm.generate_key_pair({
        user_id: params.userId,
        passphrase: params.passphrase,
      });

      const keyPair: RpgpKeyPair = {
        keyId: result.key_id,
        fingerprint: result.fingerprint,
        userId: result.user_id,
        algorithm: result.algorithm,
        publicKeyArmored: result.public_key_armored,
        privateKeyArmored: result.private_key_armored,
        createdAt: new Date(result.created_at),
      };

      // Store the key pair in browser storage
      await storageService.storeKeyPair(keyPair, params.passphrase);

      return keyPair;
    } catch (error) {
      console.error('Key generation failed:', error);
      throw new Error(`Failed to generate key pair: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async getPublicKey(keyId: string): Promise<RpgpPublicKey | undefined> {
    // First try to get from local storage
    const storedKey = storageService.getPublicKey(keyId);
    if (storedKey) {
      return storedKey;
    }

    // If not in storage, try WASM module (for keys generated in current session)
    await this.initializeWasm();
    
    if (!this.wasm) {
      return undefined;
    }

    try {
      const result = await this.wasm.get_public_key(keyId);
      
      return {
        keyId: result.key_id,
        fingerprint: result.fingerprint,
        userId: result.user_id,
        algorithm: result.algorithm,
        publicKeyArmored: result.public_key_armored,
        createdAt: new Date(result.created_at),
      };
    } catch {
      return undefined;
    }
  }

  async getAllPublicKeys(): Promise<RpgpPublicKey[]> {
    // Get keys from storage
    const storedKeys = storageService.getAllPublicKeys();
    
    // Also get any keys from current WASM session
    await this.initializeWasm();
    
    if (this.wasm) {
      try {
        const wasmKeys = await this.wasm.get_all_public_keys();
        const wasmKeyList: RpgpPublicKey[] = wasmKeys.map((result: any) => ({
          keyId: result.key_id,
          fingerprint: result.fingerprint,
          userId: result.user_id,
          algorithm: result.algorithm,
          publicKeyArmored: result.public_key_armored,
          createdAt: new Date(result.created_at),
        }));

        // Merge and deduplicate
        const allKeys = [...storedKeys];
        for (const wasmKey of wasmKeyList) {
          if (!allKeys.find(k => k.keyId === wasmKey.keyId)) {
            allKeys.push(wasmKey);
          }
        }
        
        return allKeys;
      } catch (error) {
        console.warn('Failed to get keys from WASM session:', error);
      }
    }
    
    return storedKeys;
  }

  async encryptMessage(params: EncryptParams): Promise<string> {
    await this.initializeWasm();
    
    if (!this.wasm) {
      throw new Error('RPGP WebAssembly module not initialized');
    }

    if (!params.plaintext.trim()) {
      throw new Error('Message cannot be empty');
    }

    if (params.recipientKeyIds.length === 0) {
      throw new Error('At least one recipient key is required');
    }

    try {
      const ciphertext = await this.wasm.encrypt_message({
        recipient_key_ids: params.recipientKeyIds,
        plaintext: params.plaintext,
      });

      return ciphertext;
    } catch (error) {
      console.error('Encryption failed:', error);
      throw new Error(`Failed to encrypt message: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async decryptMessage(params: DecryptParams): Promise<string> {
    await this.initializeWasm();
    
    if (!this.wasm) {
      throw new Error('RPGP WebAssembly module not initialized');
    }

    if (!params.ciphertext.trim()) {
      throw new Error('Ciphertext cannot be empty');
    }

    if (!params.passphrase) {
      throw new Error('Passphrase is required for decryption');
    }

    // Check if we have the private key in storage
    if (!storageService.hasPrivateKey(params.privateKeyId)) {
      throw new Error('Private key not found. You may need to import the private key first.');
    }

    try {
      // Retrieve the private key from storage
      const privateKeyArmored = await storageService.retrievePrivateKey(params.privateKeyId, params.passphrase);
      
      if (!privateKeyArmored) {
        throw new Error('Failed to decrypt private key. Please check your passphrase.');
      }

      const plaintext = await this.wasm.decrypt_message({
        private_key_id: params.privateKeyId,
        passphrase: params.passphrase,
        ciphertext: params.ciphertext,
      });

      return plaintext;
    } catch (error) {
      console.error('Decryption failed:', error);
      if (error instanceof Error && error.message.includes('passphrase')) {
        throw new Error('Incorrect passphrase or corrupted private key');
      }
      throw new Error(`Failed to decrypt message: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async signMessage(params: SignParams): Promise<string> {
    await this.initializeWasm();
    
    if (!this.wasm) {
      throw new Error('RPGP WebAssembly module not initialized');
    }

    if (!params.message.trim()) {
      throw new Error('Message cannot be empty');
    }

    if (!params.passphrase) {
      throw new Error('Passphrase is required for signing');
    }

    // Check if we have the private key
    if (!storageService.hasPrivateKey(params.privateKeyId)) {
      throw new Error('Private key not found. You may need to import the private key first.');
    }

    try {
      // Retrieve the private key from storage
      const privateKeyArmored = await storageService.retrievePrivateKey(params.privateKeyId, params.passphrase);
      
      if (!privateKeyArmored) {
        throw new Error('Failed to decrypt private key. Please check your passphrase.');
      }

      const signedMessage = await this.wasm.sign_message({
        private_key_id: params.privateKeyId,
        passphrase: params.passphrase,
        message: params.message,
      });

      return signedMessage;
    } catch (error) {
      console.error('Signing failed:', error);
      if (error instanceof Error && error.message.includes('passphrase')) {
        throw new Error('Incorrect passphrase or corrupted private key');
      }
      throw new Error(`Failed to sign message: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async createDetachedSignature(params: SignParams): Promise<string> {
    await this.initializeWasm();
    
    if (!this.wasm) {
      throw new Error('RPGP WebAssembly module not initialized');
    }

    if (!params.message.trim()) {
      throw new Error('Message cannot be empty');
    }

    if (!params.passphrase) {
      throw new Error('Passphrase is required for signing');
    }

    // Check if we have the private key
    if (!storageService.hasPrivateKey(params.privateKeyId)) {
      throw new Error('Private key not found. You may need to import the private key first.');
    }

    try {
      // Retrieve the private key from storage
      const privateKeyArmored = await storageService.retrievePrivateKey(params.privateKeyId, params.passphrase);
      
      if (!privateKeyArmored) {
        throw new Error('Failed to decrypt private key. Please check your passphrase.');
      }

      const signature = await this.wasm.create_detached_signature({
        private_key_id: params.privateKeyId,
        passphrase: params.passphrase,
        message: params.message,
      });

      return signature;
    } catch (error) {
      console.error('Detached signature creation failed:', error);
      if (error instanceof Error && error.message.includes('passphrase')) {
        throw new Error('Incorrect passphrase or corrupted private key');
      }
      throw new Error(`Failed to create detached signature: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async verifyMessage(params: VerifyParams): Promise<{isValid: boolean, message: string}> {
    await this.initializeWasm();
    
    if (!this.wasm) {
      throw new Error('RPGP WebAssembly module not initialized');
    }

    if (!params.message.trim() || !params.signature.trim()) {
      throw new Error('Both message and signature are required');
    }

    try {
      const result = await this.wasm.verify_message({
        signer_key_id: params.signerKeyId,
        message: params.message,
        signature: params.signature,
      });

      return {
        isValid: result.is_valid,
        message: result.message,
      };
    } catch (error) {
      console.error('Verification failed:', error);
      return {
        isValid: false,
        message: `Verification failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
      };
    }
  }

  // Additional utility methods
  async isInitialized(): Promise<boolean> {
    try {
      await this.initializeWasm();
      return this.wasm !== null;
    } catch {
      return false;
    }
  }

  getStorageInfo() {
    return storageService.getStorageInfo();
  }

  clearAllKeys(): void {
    storageService.clearAllKeys();
  }
}

export const rpgpService = new RPGPService();