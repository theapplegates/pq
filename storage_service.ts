import { RpgpPublicKey, RpgpKeyPair } from '../types';

const STORAGE_KEYS = {
  PUBLIC_KEYS: 'rpgp_public_keys',
  PRIVATE_KEYS: 'rpgp_private_keys', // Store encrypted private key data
  KEY_METADATA: 'rpgp_key_metadata',
} as const;

interface StoredKeyMetadata {
  keyId: string;
  fingerprint: string;
  userId: string;
  algorithm: string;
  createdAt: string;
  hasPrivateKey: boolean;
}

interface StoredPrivateKey {
  keyId: string;
  encryptedPrivateKey: string; // Base64 encoded encrypted private key
  salt: string; // For key derivation
}

class StorageService {
  private isAvailable(): boolean {
    try {
      const test = '__storage_test__';
      localStorage.setItem(test, test);
      localStorage.removeItem(test);
      return true;
    } catch {
      return false;
    }
  }

  private async encryptPrivateKey(privateKeyArmored: string, passphrase: string): Promise<{ encrypted: string; salt: string }> {
    // Simple encryption for storage - in production, use a proper encryption library
    const encoder = new TextEncoder();
    const data = encoder.encode(privateKeyArmored);
    const salt = crypto.getRandomValues(new Uint8Array(16));
    
    // Derive key from passphrase
    const keyMaterial = await crypto.subtle.importKey(
      'raw',
      encoder.encode(passphrase),
      { name: 'PBKDF2' },
      false,
      ['deriveKey']
    );
    
    const key = await crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt: salt,
        iterations: 100000,
        hash: 'SHA-256',
      },
      keyMaterial,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt']
    );
    
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encrypted = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv: iv },
      key,
      data
    );
    
    // Combine IV and encrypted data
    const combined = new Uint8Array(iv.length + encrypted.byteLength);
    combined.set(iv);
    combined.set(new Uint8Array(encrypted), iv.length);
    
    return {
      encrypted: btoa(String.fromCharCode(...combined)),
      salt: btoa(String.fromCharCode(...salt)),
    };
  }

  private async decryptPrivateKey(encryptedData: string, salt: string, passphrase: string): Promise<string> {
    const encoder = new TextEncoder();
    const decoder = new TextDecoder();
    
    // Decode base64
    const combined = new Uint8Array(atob(encryptedData).split('').map(c => c.charCodeAt(0)));
    const saltBytes = new Uint8Array(atob(salt).split('').map(c => c.charCodeAt(0)));
    
    // Extract IV and encrypted data
    const iv = combined.slice(0, 12);
    const encrypted = combined.slice(12);
    
    // Derive key from passphrase
    const keyMaterial = await crypto.subtle.importKey(
      'raw',
      encoder.encode(passphrase),
      { name: 'PBKDF2' },
      false,
      ['deriveKey']
    );
    
    const key = await crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt: saltBytes,
        iterations: 100000,
        hash: 'SHA-256',
      },
      keyMaterial,
      { name: 'AES-GCM', length: 256 },
      false,
      ['decrypt']
    );
    
    const decrypted = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv: iv },
      key,
      encrypted
    );
    
    return decoder.decode(decrypted);
  }

  async storeKeyPair(keyPair: RpgpKeyPair, passphrase?: string): Promise<void> {
    if (!this.isAvailable()) {
      throw new Error('Browser storage is not available');
    }

    try {
      // Store public key metadata
      const metadata: StoredKeyMetadata = {
        keyId: keyPair.keyId,
        fingerprint: keyPair.fingerprint,
        userId: keyPair.userId,
        algorithm: keyPair.algorithm,
        createdAt: keyPair.createdAt.toISOString(),
        hasPrivateKey: true,
      };

      const existingMetadata = this.getStoredMetadata();
      const updatedMetadata = existingMetadata.filter(m => m.keyId !== keyPair.keyId);
      updatedMetadata.push(metadata);
      
      localStorage.setItem(STORAGE_KEYS.KEY_METADATA, JSON.stringify(updatedMetadata));

      // Store public key
      const existingPublicKeys = this.getStoredPublicKeys();
      const updatedPublicKeys = existingPublicKeys.filter(k => k.keyId !== keyPair.keyId);
      const { privateKeyArmored, ...publicKey } = keyPair;
      updatedPublicKeys.push(publicKey);
      
      localStorage.setItem(STORAGE_KEYS.PUBLIC_KEYS, JSON.stringify(updatedPublicKeys));

      // Store encrypted private key if passphrase is provided
      if (passphrase && keyPair.privateKeyArmored) {
        const { encrypted, salt } = await this.encryptPrivateKey(keyPair.privateKeyArmored, passphrase);
        
        const storedPrivateKey: StoredPrivateKey = {
          keyId: keyPair.keyId,
          encryptedPrivateKey: encrypted,
          salt,
        };

        const existingPrivateKeys = this.getStoredPrivateKeys();
        const updatedPrivateKeys = existingPrivateKeys.filter(k => k.keyId !== keyPair.keyId);
        updatedPrivateKeys.push(storedPrivateKey);
        
        localStorage.setItem(STORAGE_KEYS.PRIVATE_KEYS, JSON.stringify(updatedPrivateKeys));
      }
    } catch (error) {
      throw new Error(`Failed to store key pair: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async retrievePrivateKey(keyId: string, passphrase: string): Promise<string | null> {
    if (!this.isAvailable()) {
      throw new Error('Browser storage is not available');
    }

    try {
      const privateKeys = this.getStoredPrivateKeys();
      const storedKey = privateKeys.find(k => k.keyId === keyId);
      
      if (!storedKey) {
        return null;
      }

      return await this.decryptPrivateKey(storedKey.encryptedPrivateKey, storedKey.salt, passphrase);
    } catch (error) {
      throw new Error(`Failed to retrieve private key: ${error instanceof Error ? error.message : 'Invalid passphrase'}`);
    }
  }

  getAllPublicKeys(): RpgpPublicKey[] {
    if (!this.isAvailable()) {
      return [];
    }

    try {
      return this.getStoredPublicKeys().map(key => ({
        ...key,
        createdAt: new Date(key.createdAt),
      }));
    } catch {
      return [];
    }
  }

  getPublicKey(keyId: string): RpgpPublicKey | null {
    const keys = this.getAllPublicKeys();
    return keys.find(k => k.keyId === keyId) || null;
  }

  hasPrivateKey(keyId: string): boolean {
    const metadata = this.getStoredMetadata();
    const keyMetadata = metadata.find(m => m.keyId === keyId);
    return keyMetadata?.hasPrivateKey || false;
  }

  deleteKey(keyId: string): void {
    if (!this.isAvailable()) {
      return;
    }

    try {
      // Remove from metadata
      const metadata = this.getStoredMetadata();
      const updatedMetadata = metadata.filter(m => m.keyId !== keyId);
      localStorage.setItem(STORAGE_KEYS.KEY_METADATA, JSON.stringify(updatedMetadata));

      // Remove public key
      const publicKeys = this.getStoredPublicKeys();
      const updatedPublicKeys = publicKeys.filter(k => k.keyId !== keyId);
      localStorage.setItem(STORAGE_KEYS.PUBLIC_KEYS, JSON.stringify(updatedPublicKeys));

      // Remove private key
      const privateKeys = this.getStoredPrivateKeys();
      const updatedPrivateKeys = privateKeys.filter(k => k.keyId !== keyId);
      localStorage.setItem(STORAGE_KEYS.PRIVATE_KEYS, JSON.stringify(updatedPrivateKeys));
    } catch (error) {
      console.error('Failed to delete key:', error);
    }
  }

  clearAllKeys(): void {
    if (!this.isAvailable()) {
      return;
    }

    try {
      localStorage.removeItem(STORAGE_KEYS.KEY_METADATA);
      localStorage.removeItem(STORAGE_KEYS.PUBLIC_KEYS);
      localStorage.removeItem(STORAGE_KEYS.PRIVATE_KEYS);
    } catch (error) {
      console.error('Failed to clear keys:', error);
    }
  }

  private getStoredMetadata(): StoredKeyMetadata[] {
    try {
      const stored = localStorage.getItem(STORAGE_KEYS.KEY_METADATA);
      return stored ? JSON.parse(stored) : [];
    } catch {
      return [];
    }
  }

  private getStoredPublicKeys(): RpgpPublicKey[] {
    try {
      const stored = localStorage.getItem(STORAGE_KEYS.PUBLIC_KEYS);
      return stored ? JSON.parse(stored) : [];
    } catch {
      return [];
    }
  }

  private getStoredPrivateKeys(): StoredPrivateKey[] {
    try {
      const stored = localStorage.getItem(STORAGE_KEYS.PRIVATE_KEYS);
      return stored ? JSON.parse(stored) : [];
    } catch {
      return [];
    }
  }

  // Get storage usage information
  getStorageInfo(): { used: number; available: boolean; keyCount: number } {
    if (!this.isAvailable()) {
      return { used: 0, available: false, keyCount: 0 };
    }

    let used = 0;
    try {
      for (const key of Object.values(STORAGE_KEYS)) {
        const item = localStorage.getItem(key);
        if (item) {
          used += item.length;
        }
      }
    } catch {
      // Ignore errors
    }

    return {
      used,
      available: true,
      keyCount: this.getStoredMetadata().length,
    };
  }
}

export const storageService = new StorageService();
    