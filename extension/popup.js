document.addEventListener('DOMContentLoaded', async () => {
  const toggleAuto = document.getElementById('toggleAutoIntercept');
  const toggleDialog = document.getElementById('toggleShowDialog');
  const toggleAll = document.getElementById('toggleInterceptAll');
  const toggleSniffer = document.getElementById('toggleMediaSniffer');
  const selectConn = document.getElementById('selectConnections');
  const inputExts = document.getElementById('inputExtensions');
  const statusBadge = document.getElementById('statusBadge');
  const statusText = document.getElementById('statusText');
  const activeCountEl = document.getElementById('activeTasksCount');
  const currentSpeedEl = document.getElementById('currentSpeed');
  const btnOpen = document.getElementById('btnOpenDashboard');

  // Load stored settings
  const config = await chrome.storage.local.get({
    autoIntercept: true,
    showDialog: true,
    interceptAll: false,
    mediaSniffer: true,
    connections: 64,
    serverUrl: 'http://127.0.0.1:6800',
    extensions: [
      'html', 'htm', 'xhtml', 'mhtml', 'pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx',
      'txt', 'rtf', 'csv', 'epub', 'mobi', 'azw3', 'odt', 'ods', 'odp', 'xml', 'json',
      'zip', 'rar', '7z', 'tar', 'gz', 'tgz', 'bz2', 'tbz2', 'xz', 'txz', 'z', 'lz', 'lzma', 'lzh', 'cab', 'arj', 'wim',
      'iso', 'img', 'dmg', 'vhd', 'vhdx', 'vdi', 'qcow2', 'nrg', 'cue', 'bin',
      'exe', 'msi', 'deb', 'rpm', 'apk', 'pkg', 'appimage', 'flatpak', 'snap', 'run', 'sh', 'bat', 'cmd', 'ps1', 'jar', 'war', 'ipa',
      'mp4', 'mkv', 'avi', 'mov', 'flv', 'wmv', 'webm', 'm4v', '3gp', 'ts', 'm2ts', 'vob', 'f4v', 'rm', 'rmvb', 'asf', 'ogv',
      'mp3', 'flac', 'wav', 'aac', 'ogg', 'm4a', 'opus', 'ape', 'alac', 'mid', 'midi', 'wma',
      'png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'bmp', 'ico', 'tiff', 'tif', 'psd', 'ai', 'raw', 'cr2', 'nef',
      'torrent', 'dat', 'db', 'sqlite', 'sql', 'bak'
    ]
  });

  toggleAuto.checked = config.autoIntercept;
  if (toggleDialog) toggleDialog.checked = config.showDialog !== false;
  toggleAll.checked = config.interceptAll;
  if (toggleSniffer) toggleSniffer.checked = config.mediaSniffer !== false;
  selectConn.value = config.connections.toString();
  inputExts.value = config.extensions.join(', ');
  if (window.initCustomSelects) {
    window.initCustomSelects();
  }

  // ========================================================
  // Dynamic Accent Theme Engine (Apple Fluid / Material Design)
  // ========================================================
  function hexToRgb(hex) {
    hex = hex.replace('#', '').trim();
    if (hex.length === 3) {
      hex = hex.split('').map(c => c + c).join('');
    }
    const num = parseInt(hex, 16);
    if (isNaN(num)) return '14, 165, 233';
    return `${(num >> 16) & 255}, ${(num >> 8) & 255}, ${num & 255}`;
  }

  function applyTheme(hexColor, name = '') {
    if (!hexColor) return;
    const rgb = hexToRgb(hexColor);
    const root = document.documentElement;
    root.style.setProperty('--theme-accent', hexColor);
    root.style.setProperty('--theme-accent-rgb', rgb);
    root.style.setProperty('--theme-accent-glow', `rgba(${rgb}, 0.4)`);
    root.style.setProperty('--theme-accent-gradient', `linear-gradient(135deg, rgba(${rgb}, 0.95) 0%, rgba(${rgb}, 0.7) 100%)`);

    document.querySelectorAll('.theme-swatch-mini').forEach(el => {
      if (el.dataset.color.toLowerCase() === hexColor.toLowerCase()) {
        el.classList.add('is-active');
      } else {
        el.classList.remove('is-active');
      }
    });

    try {
      chrome.storage.local.set({ dpls_theme_accent: hexColor, dpls_theme_name: name });
    } catch (e) {}
  }

  const themeConfig = await chrome.storage.local.get(['dpls_theme_accent', 'dpls_theme_name']);
  if (themeConfig.dpls_theme_accent) {
    applyTheme(themeConfig.dpls_theme_accent, themeConfig.dpls_theme_name);
  }

  if (typeof chrome !== 'undefined' && chrome.storage && chrome.storage.onChanged) {
    chrome.storage.onChanged.addListener((changes, area) => {
      if (area === 'local' && changes.dpls_theme_accent) {
        applyTheme(changes.dpls_theme_accent.newValue);
      }
    });
  }

  document.querySelectorAll('.theme-swatch-mini').forEach(el => {
    el.addEventListener('click', () => {
      const color = el.dataset.color;
      const title = el.getAttribute('title') || '';
      applyTheme(color, title);
    });
  });

  // Save changes
  toggleAuto.addEventListener('change', () => {
    chrome.storage.local.set({ autoIntercept: toggleAuto.checked });
  });

  if (toggleDialog) {
    toggleDialog.addEventListener('change', () => {
      chrome.storage.local.set({ showDialog: toggleDialog.checked });
    });
  }

  toggleAll.addEventListener('change', () => {
    chrome.storage.local.set({ interceptAll: toggleAll.checked });
  });

  if (toggleSniffer) {
    toggleSniffer.addEventListener('change', () => {
      chrome.storage.local.set({ mediaSniffer: toggleSniffer.checked });
    });
  }

  selectConn.addEventListener('change', () => {
    chrome.storage.local.set({ connections: parseInt(selectConn.value, 10) });
  });

  inputExts.addEventListener('change', () => {
    const exts = inputExts.value
      .split(',')
      .map(s => s.trim().toLowerCase())
      .filter(s => s.length > 0);
    chrome.storage.local.set({ extensions: exts });
  });

  btnOpen.addEventListener('click', () => {
    chrome.tabs.create({ url: config.serverUrl });
  });

  // Check live connection & stats
  async function updateStatus() {
    try {
      const res = await fetch(`${config.serverUrl}/api/tasks`, { cache: 'no-store' });
      if (res.ok) {
        statusBadge.className = 'status-indicator status-connected';
        statusText.innerText = '已連線';

        const tasks = await res.json();
        let totalSpeed = 0;
        let activeCount = 0;

        tasks.forEach(t => {
          if (t.state === 'Downloading') {
            activeCount++;
            totalSpeed += t.speed_bps || 0;
          }
        });

        activeCountEl.innerText = activeCount.toString();
        const speedMB = (totalSpeed / (1024 * 1024)).toFixed(2);
        currentSpeedEl.innerText = `${speedMB} MB/s`;
        return;
      }
    } catch (e) {}

    statusBadge.className = 'status-indicator status-disconnected';
    statusText.innerText = '未連線';
    activeCountEl.innerText = '0';
    currentSpeedEl.innerText = '0.0 MB/s';
  }

  updateStatus();
  setInterval(updateStatus, 1500);
});
