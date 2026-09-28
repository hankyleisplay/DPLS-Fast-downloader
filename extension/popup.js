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
    extensions: ['zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'iso', 'img', 'dmg', 'exe', 'msi', 'deb', 'rpm', 'apk', 'pkg', 'mp4', 'mkv', 'avi', 'mov', 'flv', 'wmv', 'webm', 'mp3', 'flac', 'wav', 'aac', 'ogg', 'pdf', 'epub', 'bin', 'torrent']
  });

  toggleAuto.checked = config.autoIntercept;
  if (toggleDialog) toggleDialog.checked = config.showDialog !== false;
  toggleAll.checked = config.interceptAll;
  if (toggleSniffer) toggleSniffer.checked = config.mediaSniffer !== false;
  selectConn.value = config.connections.toString();
  inputExts.value = config.extensions.join(', ');

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
