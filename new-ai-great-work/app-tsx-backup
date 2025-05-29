import React, { useState, useCallback } from 'react';
import { KeyManagementSection } from './components/KeyManagementSection';
import { EncryptSection } from './components/EncryptSection';
import { DecryptSection } from './components/DecryptSection';
import { SignSection } from './components/SignSection';
import { VerifySection } from './components/VerifySection';
import { CryptoHelper } from './components/CryptoHelper';
import { WasmStatus } from './components/WasmStatus';
import { Tabs, Tab } from './components/common/Tabs';
import { RpgpPublicKey } from './types';
import { KeyIcon, LockClosedIcon, LockOpenIcon, PencilSquareIcon, CheckBadgeIcon, QuestionMarkCircleIcon, SunIcon, MoonIcon } from '@heroicons/react/24/outline';

enum Section {
  KEYS = 'Key Management',
  ENCRYPT = 'Encrypt',
  DECRYPT = 'Decrypt',
  SIGN = 'Sign',
  VERIFY = 'Verify',
  HELPER = 'Crypto Helper',
}

const App: React.FC = () => {
  const [activeSection, setActiveSection] = useState<Section>(Section.KEYS);
  const [keys, setKeys] = useState<RpgpPublicKey[]>([]);
  const [isWasmReady, setIsWasmReady] = useState(false);
  const [isDarkMode, setIsDarkMode] = useState(() => {
    if (typeof window !== 'undefined') {
      return localStorage.get