import React, { useState, useEffect } from 'react';
import { Alert } from './common/Alert';
import { Spinner } from './common/Spinner';
import { rpgpService } from '../services/rpgpService';

interface WasmStatusProps {
  onStatusChange?: (isReady: boolean) => void;
}

export const WasmStatus: React.FC<WasmStatusProps> = ({ onStatusChange }) => {
  const [isLoading, setIsLoading] = useState(true);
  const [isReady, setIsReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const checkWasmStatus = async () => {
      try {
        setIsLoading(true);
        setError(null);
        
        const ready = await rpgpService.isInitialized();
        
        setIsReady(ready);
        setIsLoading(false);
        
        if (onStatusChange) {
          onStatusChange(ready);
        }
        
        if (!ready) {
          setError('WebAssembly module failed to initialize. Please check that the WASM files are built and accessible.');
        }
      } catch (err) {
        setIsLoading(false);
        setIsReady(false);
        const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred during WASM initialization';
        setError(errorMessage);
        
        if (onStatusChange) {
          onStatusChange(false);
        }
      }
    };

    checkWasmStatus();
  }, [onStatusChange]);

  if (isLoading) {
    return (
      <Alert 
        type="info" 
        title="Initializing Cryptography Engine"
        message={
          <div className="flex items-center space-x-3">
            <Spinner size="sm" />
            <span>Loading post-quantum cryptography WebAssembly module...</span>
          </div>
        }
      />
    );
  }

  if (error) {
    return (
      <Alert 
        type="error" 
        title="Cryptography Engine Error"
        message={
          <div>
            <p className="mb-2">{error}</p>
            <p className="text-sm">
              <strong>Setup required:</strong> Please run <code className="bg-red-100 dark:bg-red-800 px-1 rounded">npm run setup</code> to build the WebAssembly module with post-quantum cryptography support.
            </p>
          </div>
        }
      />
    );
  }

  if (isReady) {
    const storageInfo = rpgpService.getStorageInfo();
    
    return (
      <Alert 
        type="success" 
        title="Real Post-Quantum Cryptography Ready"
        message={
          <div>
            <p className="mb-1">✅ Dilithium5 (signatures) and Kyber1024 (encryption) are active</p>
            <p className="text-xs text-green-600 dark:text-green-300">
              Storage: {storageInfo.keyCount} keys, {Math.round(storageInfo.used / 1024)}KB used
            </p>
          </div>
        }
      />
    );
  }

  return null;
};
