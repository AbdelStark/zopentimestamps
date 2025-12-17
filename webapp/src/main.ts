import './styles/main.css';
import init, {
  hash_bytes,
  hash_hex,
  create_proof,
  parse_proof,
  serialize_proof,
  encode_proof,
  decode_proof,
  is_compact_format,
  WasmHashAlgorithm,
} from '../pkg/zots_wasm.js';
import QRCode from 'qrcode';
import { Html5Qrcode } from 'html5-qrcode';

// Types
interface Settings {
  explorerUrl: string;
  network: 'testnet' | 'mainnet';
  nostrViewerUrl: string;
  defaultAlgorithm: 'sha256' | 'blake3';
}

// State
let currentHash: string | null = null;
let currentProof: any = null;
let qrScanner: Html5Qrcode | null = null;
let settings: Settings = loadSettings();

// Default settings
function getDefaultSettings(): Settings {
  return {
    explorerUrl: 'https://blockexplorer.one/zcash/testnet',
    network: 'testnet',
    nostrViewerUrl: 'https://iris.to',
    defaultAlgorithm: 'sha256',
  };
}

// Load settings from localStorage
function loadSettings(): Settings {
  const saved = localStorage.getItem('zots-settings');
  if (saved) {
    try {
      return { ...getDefaultSettings(), ...JSON.parse(saved) };
    } catch {
      return getDefaultSettings();
    }
  }
  return getDefaultSettings();
}

// Save settings to localStorage
function saveSettings(newSettings: Settings) {
  settings = newSettings;
  localStorage.setItem('zots-settings', JSON.stringify(settings));
}

// Toast notifications
function showToast(message: string, type: 'success' | 'error' | 'info' = 'info') {
  const container = document.getElementById('toast-container')!;
  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.innerHTML = `
    <span>${type === 'success' ? '✓' : type === 'error' ? '✗' : 'ℹ'}</span>
    <span>${message}</span>
  `;
  container.appendChild(toast);
  setTimeout(() => toast.remove(), 3000);
}

// Copy to clipboard
async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    showToast('Copied to clipboard!', 'success');
  } catch {
    showToast('Failed to copy', 'error');
  }
}

// Navigation
function showView(viewId: string) {
  document.querySelectorAll('.view').forEach(v => v.classList.add('hidden'));
  document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));

  const view = document.getElementById(`view-${viewId}`);
  const navBtn = document.getElementById(`nav-${viewId}`);

  if (view) view.classList.remove('hidden');
  if (navBtn) navBtn.classList.add('active');
}

// Get selected algorithm
function getSelectedAlgorithm(): WasmHashAlgorithm {
  const selected = document.querySelector('input[name="algorithm"]:checked') as HTMLInputElement;
  return selected?.value === 'blake3' ? WasmHashAlgorithm.Blake3 : WasmHashAlgorithm.Sha256;
}

// Hash a file
async function hashFile(file: File): Promise<string> {
  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  const algorithm = getSelectedAlgorithm();
  return hash_bytes(bytes, algorithm);
}

// Initialize WASM and app
async function initApp() {
  try {
    await init();
    console.log('WASM initialized successfully');
    initializeUI();
  } catch (error) {
    console.error('Failed to initialize WASM:', error);
    showToast('Failed to initialize. Please refresh.', 'error');
  }
}

// Initialize UI event listeners
function initializeUI() {
  // Navigation
  document.getElementById('nav-hash')?.addEventListener('click', () => showView('hash'));
  document.getElementById('nav-encode')?.addEventListener('click', () => showView('encode'));
  document.getElementById('nav-decode')?.addEventListener('click', () => showView('decode'));
  document.getElementById('nav-qr')?.addEventListener('click', () => showView('qr'));
  document.getElementById('nav-settings')?.addEventListener('click', () => {
    showView('settings');
    loadSettingsUI();
  });

  // File upload / drop zone
  const dropZone = document.getElementById('drop-zone')!;
  const fileInput = document.getElementById('file-input') as HTMLInputElement;
  const hashBtn = document.getElementById('hash-btn') as HTMLButtonElement;
  const hashInput = document.getElementById('hash-input') as HTMLInputElement;

  dropZone.addEventListener('click', () => fileInput.click());

  dropZone.addEventListener('dragover', (e) => {
    e.preventDefault();
    dropZone.classList.add('border-zots-primary', 'bg-zots-primary/5');
  });

  dropZone.addEventListener('dragleave', () => {
    dropZone.classList.remove('border-zots-primary', 'bg-zots-primary/5');
  });

  dropZone.addEventListener('drop', async (e) => {
    e.preventDefault();
    dropZone.classList.remove('border-zots-primary', 'bg-zots-primary/5');
    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      await handleFileSelect(files[0]);
    }
  });

  fileInput.addEventListener('change', async () => {
    if (fileInput.files && fileInput.files.length > 0) {
      await handleFileSelect(fileInput.files[0]);
    }
  });

  // Hash input
  hashInput.addEventListener('input', () => {
    hashBtn.disabled = !hashInput.value.trim() && !fileInput.files?.length;
  });

  // Hash button
  hashBtn.addEventListener('click', async () => {
    if (hashInput.value.trim()) {
      try {
        const algorithm = getSelectedAlgorithm();
        currentHash = hash_hex(hashInput.value.trim(), algorithm);
        showHashResult(currentHash);
      } catch (error: any) {
        showToast(error.message || 'Invalid hash input', 'error');
      }
    }
  });

  // Copy hash
  document.getElementById('copy-hash')?.addEventListener('click', () => {
    if (currentHash) copyToClipboard(currentHash);
  });

  // Create proof from hash
  document.getElementById('create-proof-btn')?.addEventListener('click', () => {
    if (currentHash) {
      try {
        const algorithm = getSelectedAlgorithm();
        currentProof = create_proof(currentHash, algorithm);
        showView('encode');
        (document.getElementById('encode-input') as HTMLTextAreaElement).value = serialize_proof(currentProof);
        document.getElementById('encode-btn')?.click();
      } catch (error: any) {
        showToast(error.message || 'Failed to create proof', 'error');
      }
    }
  });

  // Generate QR from hash
  document.getElementById('generate-qr-btn')?.addEventListener('click', () => {
    if (currentHash) {
      showView('qr');
      (document.getElementById('qr-data-input') as HTMLTextAreaElement).value = currentHash;
      document.getElementById('generate-qr-main-btn')?.click();
    }
  });

  // Encode view
  const proofFileInput = document.getElementById('proof-file-input') as HTMLInputElement;
  proofFileInput.addEventListener('change', async () => {
    if (proofFileInput.files && proofFileInput.files.length > 0) {
      const file = proofFileInput.files[0];
      const text = await file.text();
      (document.getElementById('encode-input') as HTMLTextAreaElement).value = text;
    }
  });

  document.getElementById('encode-btn')?.addEventListener('click', () => {
    const input = (document.getElementById('encode-input') as HTMLTextAreaElement).value.trim();
    if (!input) {
      showToast('Please enter proof JSON', 'error');
      return;
    }
    try {
      const proof = parse_proof(input);
      const compact = encode_proof(proof);
      currentProof = proof;

      document.getElementById('compact-output')!.textContent = compact;
      document.getElementById('compact-size')!.textContent = `${compact.length} characters`;
      document.getElementById('encode-result')!.classList.remove('hidden');
    } catch (error: any) {
      showToast(error.message || 'Invalid proof JSON', 'error');
    }
  });

  document.getElementById('copy-compact')?.addEventListener('click', () => {
    const compact = document.getElementById('compact-output')?.textContent;
    if (compact) copyToClipboard(compact);
  });

  document.getElementById('compact-qr-btn')?.addEventListener('click', () => {
    const compact = document.getElementById('compact-output')?.textContent;
    if (compact) {
      showView('qr');
      (document.getElementById('qr-data-input') as HTMLTextAreaElement).value = compact;
      document.getElementById('generate-qr-main-btn')?.click();
    }
  });

  // Decode view
  document.getElementById('decode-btn')?.addEventListener('click', () => {
    const input = (document.getElementById('decode-input') as HTMLTextAreaElement).value.trim();
    if (!input) {
      showToast('Please enter compact proof', 'error');
      return;
    }
    try {
      const proof = decode_proof(input);
      currentProof = proof;
      displayProofInfo(proof);
      document.getElementById('decode-result')!.classList.remove('hidden');
    } catch (error: any) {
      showToast(error.message || 'Invalid compact proof', 'error');
    }
  });

  document.getElementById('copy-json')?.addEventListener('click', () => {
    if (currentProof) {
      copyToClipboard(serialize_proof(currentProof));
    }
  });

  document.getElementById('download-proof')?.addEventListener('click', () => {
    if (currentProof) {
      const json = serialize_proof(currentProof);
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `proof-${currentProof.hash.substring(0, 8)}.zots`;
      a.click();
      URL.revokeObjectURL(url);
      showToast('Proof downloaded!', 'success');
    }
  });

  // QR Code view
  document.getElementById('qr-tab-generate')?.addEventListener('click', () => {
    document.getElementById('qr-tab-generate')?.classList.add('active');
    document.getElementById('qr-tab-scan')?.classList.remove('active');
    document.getElementById('qr-generate-panel')?.classList.remove('hidden');
    document.getElementById('qr-scan-panel')?.classList.add('hidden');
    stopQRScanner();
  });

  document.getElementById('qr-tab-scan')?.addEventListener('click', () => {
    document.getElementById('qr-tab-scan')?.classList.add('active');
    document.getElementById('qr-tab-generate')?.classList.remove('active');
    document.getElementById('qr-scan-panel')?.classList.remove('hidden');
    document.getElementById('qr-generate-panel')?.classList.add('hidden');
  });

  document.getElementById('generate-qr-main-btn')?.addEventListener('click', async () => {
    const data = (document.getElementById('qr-data-input') as HTMLTextAreaElement).value.trim();
    if (!data) {
      showToast('Please enter data to encode', 'error');
      return;
    }
    try {
      const canvas = document.createElement('canvas');
      await QRCode.toCanvas(canvas, data, {
        width: 256,
        margin: 2,
        color: { dark: '#000000', light: '#ffffff' }
      });
      const qrCanvas = document.getElementById('qr-canvas')!;
      qrCanvas.innerHTML = '';
      qrCanvas.appendChild(canvas);
      document.getElementById('qr-output')!.classList.remove('hidden');
    } catch (error: any) {
      showToast('Failed to generate QR code: ' + error.message, 'error');
    }
  });

  document.getElementById('download-qr')?.addEventListener('click', () => {
    const canvas = document.querySelector('#qr-canvas canvas') as HTMLCanvasElement;
    if (canvas) {
      const url = canvas.toDataURL('image/png');
      const a = document.createElement('a');
      a.href = url;
      a.download = 'zots-qr.png';
      a.click();
      showToast('QR code downloaded!', 'success');
    }
  });

  document.getElementById('start-scan')?.addEventListener('click', async () => {
    await startQRScanner();
  });

  document.getElementById('copy-scanned')?.addEventListener('click', () => {
    const scanned = document.getElementById('scanned-output')?.textContent;
    if (scanned) copyToClipboard(scanned);
  });

  // Settings
  document.getElementById('save-settings')?.addEventListener('click', () => {
    const newSettings: Settings = {
      explorerUrl: (document.getElementById('setting-explorer') as HTMLInputElement).value || getDefaultSettings().explorerUrl,
      network: (document.getElementById('setting-network') as HTMLSelectElement).value as 'testnet' | 'mainnet',
      nostrViewerUrl: (document.getElementById('setting-nostr-viewer') as HTMLInputElement).value || getDefaultSettings().nostrViewerUrl,
      defaultAlgorithm: (document.getElementById('setting-algorithm') as HTMLSelectElement).value as 'sha256' | 'blake3',
    };
    saveSettings(newSettings);
    showToast('Settings saved!', 'success');
  });

  document.getElementById('reset-settings')?.addEventListener('click', () => {
    saveSettings(getDefaultSettings());
    loadSettingsUI();
    showToast('Settings reset to defaults', 'info');
  });

  // Apply default algorithm from settings
  if (settings.defaultAlgorithm === 'blake3') {
    const blake3Radio = document.querySelector('input[value="blake3"]') as HTMLInputElement;
    if (blake3Radio) blake3Radio.checked = true;
  }
}

// Handle file selection
async function handleFileSelect(file: File) {
  const hashBtn = document.getElementById('hash-btn') as HTMLButtonElement;
  const dropZone = document.getElementById('drop-zone')!;

  dropZone.innerHTML = `
    <div class="text-gray-400">
      <svg class="w-12 h-12 mx-auto mb-4 text-zots-accent animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
      </svg>
      <p class="text-lg mb-2">${file.name}</p>
      <p class="text-sm text-gray-500">${formatFileSize(file.size)}</p>
    </div>
  `;

  hashBtn.disabled = false;
  hashBtn.onclick = async () => {
    hashBtn.disabled = true;
    hashBtn.innerHTML = '<span class="animate-pulse">Hashing...</span>';
    try {
      currentHash = await hashFile(file);
      showHashResult(currentHash);
    } catch (error: any) {
      showToast(error.message || 'Failed to hash file', 'error');
    }
    hashBtn.disabled = false;
    hashBtn.innerHTML = '<span class="mr-2">🔐</span> Calculate Hash';
  };
}

// Show hash result
function showHashResult(hash: string) {
  document.getElementById('hash-output')!.textContent = hash;
  document.getElementById('hash-result')!.classList.remove('hidden');
  showToast('Hash calculated successfully!', 'success');
}

// Display proof info
function displayProofInfo(proof: any) {
  const infoDiv = document.getElementById('proof-info')!;
  const jsonOutput = document.getElementById('json-output')!;

  let html = `
    <div class="grid grid-cols-2 gap-2">
      <span class="text-gray-400">Version:</span>
      <span>${proof.version}</span>
      <span class="text-gray-400">Algorithm:</span>
      <span>${proof.hash_algorithm?.toUpperCase() || 'SHA-256'}</span>
      <span class="text-gray-400">Hash:</span>
      <span class="text-zots-accent break-all">${proof.hash}</span>
      <span class="text-gray-400">Attestations:</span>
      <span>${proof.attestations?.length || 0}</span>
    </div>
  `;

  if (proof.attestations && proof.attestations.length > 0) {
    html += '<div class="mt-4 space-y-3">';
    proof.attestations.forEach((att: any, i: number) => {
      const explorerUrl = `${settings.explorerUrl}/tx/${att.txid}`;
      const time = new Date(att.block_time * 1000).toLocaleString();
      html += `
        <div class="bg-black/20 rounded p-3">
          <div class="font-medium text-zots-primary mb-2">Attestation #${i + 1}</div>
          <div class="grid grid-cols-2 gap-1 text-sm">
            <span class="text-gray-400">Network:</span>
            <span>${att.network}</span>
            <span class="text-gray-400">Block:</span>
            <span>${att.block_height}</span>
            <span class="text-gray-400">Time:</span>
            <span>${time}</span>
            <span class="text-gray-400">TX:</span>
            <a href="${explorerUrl}" target="_blank" class="text-zots-primary hover:underline truncate">${att.txid.substring(0, 16)}...</a>
          </div>
        </div>
      `;
    });
    html += '</div>';
  }

  infoDiv.innerHTML = html;
  jsonOutput.textContent = JSON.stringify(proof, null, 2);
}

// Load settings into UI
function loadSettingsUI() {
  (document.getElementById('setting-explorer') as HTMLInputElement).value = settings.explorerUrl;
  (document.getElementById('setting-network') as HTMLSelectElement).value = settings.network;
  (document.getElementById('setting-nostr-viewer') as HTMLInputElement).value = settings.nostrViewerUrl;
  (document.getElementById('setting-algorithm') as HTMLSelectElement).value = settings.defaultAlgorithm;
}

// QR Scanner
async function startQRScanner() {
  const btn = document.getElementById('start-scan') as HTMLButtonElement;

  if (qrScanner) {
    await stopQRScanner();
    btn.innerHTML = '<span class="mr-2">📸</span> Start Camera';
    return;
  }

  try {
    qrScanner = new Html5Qrcode('qr-reader');
    await qrScanner.start(
      { facingMode: 'environment' },
      { fps: 10, qrbox: { width: 250, height: 250 } },
      (decodedText) => {
        handleScannedData(decodedText);
        stopQRScanner();
      },
      () => {}
    );
    btn.innerHTML = '<span class="mr-2">⏹</span> Stop Camera';
  } catch (error: any) {
    showToast('Failed to start camera: ' + error.message, 'error');
  }
}

async function stopQRScanner() {
  if (qrScanner) {
    try {
      await qrScanner.stop();
    } catch {}
    qrScanner = null;
  }
}

function handleScannedData(data: string) {
  document.getElementById('scanned-output')!.textContent = data;
  document.getElementById('scan-result')!.classList.remove('hidden');

  const actionsDiv = document.getElementById('scan-actions')!;
  actionsDiv.innerHTML = '';

  // Check if it's a compact proof
  if (is_compact_format(data)) {
    const btn = document.createElement('button');
    btn.className = 'btn-secondary text-sm';
    btn.textContent = 'Decode Proof';
    btn.onclick = () => {
      showView('decode');
      (document.getElementById('decode-input') as HTMLTextAreaElement).value = data;
      document.getElementById('decode-btn')?.click();
    };
    actionsDiv.appendChild(btn);
  }

  // Check if it's a valid hash
  if (/^[a-fA-F0-9]{64}$/.test(data)) {
    const btn = document.createElement('button');
    btn.className = 'btn-secondary text-sm';
    btn.textContent = 'Create Proof';
    btn.onclick = () => {
      showView('hash');
      (document.getElementById('hash-input') as HTMLInputElement).value = data;
      currentHash = data;
      showHashResult(data);
    };
    actionsDiv.appendChild(btn);
  }

  showToast('QR code scanned!', 'success');
}

// Utility
function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// Start the app
initApp();
