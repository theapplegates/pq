// This file now delegates to the real rpgp service
// Keeping the same interface to maintain compatibility with existing UI components

import { rpgpService } from './rpgpService';

export const rpgpMockService = {
  generateKeyPair: rpgpService.generateKeyPair.bind(rpgpService),
  getPublicKey: rpgpService.getPublicKey.bind(rpgpService),
  getAllPublicKeys: rpgpService.getAllPublicKeys.bind(rpgpService),
  encryptMessage: rpgpService.encryptMessage.bind(rpgpService),
  decryptMessage: rpgpService.decryptMessage.bind(rpgpService),
  signMessage: rpgpService.signMessage.bind(rpgpService),
  createDetachedSignature: rpgpService.createDetachedSignature.bind(rpgpService),
  verifyMessage: rpgpService.verifyMessage.bind(rpgpService),
};