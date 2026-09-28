const DEFAULT_CONFIG = {
  autoIntercept: true,
  showDialog: true,
  interceptAll: false,
  connections: 64,
  serverUrl: 'http://127.0.0.1:6800',
  extensions: [
    'zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'iso', 'img', 'dmg',
    'exe', 'msi', 'deb', 'rpm', 'apk', 'pkg',
    'mp4', 'mkv', 'avi', 'mov', 'flv', 'wmv', 'webm',
    'mp3', 'flac', 'wav', 'aac', 'ogg',
    'pdf', 'epub', 'bin', 'torrent'
  ]
};

const interceptedDownloadIds = new Set();

// Initialize on install
chrome.runtime.onInstalled.addListener(() => {
  chrome.storage.local.get(DEFAULT_CONFIG, (stored) => {
    chrome.storage.local.set(stored);
  });

  // Create Context Menus
  chrome.contextMenus.create({
    id: 'dpls_download_link',
    title: '⚡ 使用 DPLS-Fast 極速下載此連結',
    contexts: ['link']
  });

  chrome.contextMenus.create({
    id: 'dpls_download_media',
    title: '⚡ 使用 DPLS-Fast 極速下載媒體',
    contexts: ['image', 'video', 'audio']
  });
});

// Context Menu click
chrome.contextMenus.onClicked.addListener((info, tab) => {
  let targetUrl = null;
  if (info.menuItemId === 'dpls_download_link') {
    targetUrl = info.linkUrl;
  } else if (info.menuItemId === 'dpls_download_media') {
    targetUrl = info.srcUrl;
  }

  if (targetUrl) {
    console.log('[DPLS-Fast] Context menu download:', targetUrl);
    handleDownloadTrigger(targetUrl, null, tab ? tab.url : null);
  }
});

// Listen to content script click events
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'LINK_CLICKED' && message.url) {
    console.log('[DPLS-Fast] Received LINK_CLICKED from content script:', message.url);
    handleDownloadTrigger(message.url, message.filename, message.referer);
    sendResponse({ ok: true });
  }
});

async function handleDownloadTrigger(url, filename = null, referer = null) {
  const config = await chrome.storage.local.get(DEFAULT_CONFIG);
  if (config.showDialog !== false) {
    const dialogUrl = chrome.runtime.getURL(
      `dialog.html?url=${encodeURIComponent(url)}&filename=${encodeURIComponent(filename || '')}&referer=${encodeURIComponent(referer || '')}`
    );
    try {
      const width = 640;
      const height = 590;

      let left = 200;
      let top = 100;
      try {
        const currentWin = await chrome.windows.getCurrent();
        if (currentWin && currentWin.width && currentWin.height) {
          left = Math.max(0, Math.round(currentWin.left + (currentWin.width - width) / 2));
          top = Math.max(0, Math.round(currentWin.top + (currentWin.height - height) / 2));
        }
      } catch (e) {}

      const createdWin = await chrome.windows.create({
        url: dialogUrl,
        type: 'popup',
        width,
        height,
        left,
        top,
        focused: true
      });

      if (createdWin && createdWin.id) {
        await chrome.windows.update(createdWin.id, {
          focused: true,
          drawAttention: true
        });
      }
      return;
    } catch (e) {
      console.warn('[DPLS-Fast] Failed to open popup dialog:', e);
    }
  }

  sendTaskToDPLS(url, filename, referer);
}

// CORE INTERCEPTION 1: onDeterminingFilename (Chrome standard for real filenames & headers)
chrome.downloads.onDeterminingFilename.addListener(async (downloadItem, suggest) => {
  const config = await chrome.storage.local.get(DEFAULT_CONFIG);
  if (!config.autoIntercept) {
    suggest();
    return;
  }

  if (interceptedDownloadIds.has(downloadItem.id)) {
    suggest();
    return;
  }

  const url = downloadItem.finalUrl || downloadItem.url;
  if (!url || url.startsWith('blob:') || url.startsWith('data:')) {
    suggest();
    return;
  }

  const filename = downloadItem.filename || '';
  const ext = getFileExtension(filename || url);
  const mime = (downloadItem.mime || '').toLowerCase();

  const isMatched = config.interceptAll ||
    (ext && config.extensions.includes(ext.toLowerCase())) ||
    isDownloadMimeType(mime);

  if (isMatched) {
    console.log(`[DPLS-Fast] Intercepting download: [${filename}] url: [${url}]`);
    interceptedDownloadIds.add(downloadItem.id);

    // Cancel Chrome's native download
    try {
      await chrome.downloads.cancel(downloadItem.id);
      await chrome.downloads.erase({ id: downloadItem.id });
    } catch (e) {
      console.warn('[DPLS-Fast] Cancel error:', e);
    }

    // Forward to IDM Dialog or DPLS-Fast
    handleDownloadTrigger(url, filename ? extractBasename(filename) : null);
  } else {
    suggest();
  }
});

// CORE INTERCEPTION 2: onCreated (fast-path fallback)
chrome.downloads.onCreated.addListener(async (downloadItem) => {
  const config = await chrome.storage.local.get(DEFAULT_CONFIG);
  if (!config.autoIntercept) return;

  if (interceptedDownloadIds.has(downloadItem.id)) return;

  const url = downloadItem.finalUrl || downloadItem.url;
  if (!url || url.startsWith('blob:') || url.startsWith('data:')) return;

  const ext = getFileExtension(url);
  if (ext && config.extensions.includes(ext.toLowerCase())) {
    console.log(`[DPLS-Fast] Fast-path intercepting onCreated: [${url}]`);
    interceptedDownloadIds.add(downloadItem.id);

    try {
      await chrome.downloads.cancel(downloadItem.id);
      await chrome.downloads.erase({ id: downloadItem.id });
    } catch (e) {}

    handleDownloadTrigger(url, null);
  }
});

function isDownloadMimeType(mime) {
  if (!mime) return false;
  return mime.includes('application/zip') ||
    mime.includes('application/x-zip') ||
    mime.includes('application/x-rar') ||
    mime.includes('application/x-7z') ||
    mime.includes('application/x-tar') ||
    mime.includes('application/gzip') ||
    mime.includes('application/x-iso') ||
    mime.includes('application/octet-stream') ||
    mime.startsWith('video/') ||
    mime.startsWith('audio/');
}

function getFileExtension(pathOrUrl) {
  try {
    const urlObj = new URL(pathOrUrl, 'http://dummy.com');
    const path = urlObj.pathname;
    const dot = path.lastIndexOf('.');
    if (dot !== -1) {
      return path.slice(dot + 1).toLowerCase().split('/')[0];
    }
  } catch (e) {}
  return '';
}

function extractBasename(filePath) {
  const parts = filePath.replace(/\\/g, '/').split('/');
  return parts[parts.length - 1];
}

async function sendTaskToDPLS(url, filename = null, referer = null) {
  const config = await chrome.storage.local.get(DEFAULT_CONFIG);

  const payload = {
    url,
    filename,
    connections: config.connections
  };

  console.log('[DPLS-Fast] Sending task to server:', payload);

  try {
    const res = await fetch(`${config.serverUrl}/api/add`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(payload)
    });

    if (res.ok) {
      const data = await res.json();
      console.log('[DPLS-Fast] Task added successfully:', data);
      showNotification('⚡ DPLS-Fast 已接管下載', `已由 DPLS-Fast 開始極速多線程下載：\n${filename || url}`);
    } else {
      console.error('[DPLS-Fast] Server error:', res.status);
      showNotification('❌ 下載接管失敗', `伺服器回應錯誤 (${res.status})，請檢查 DPLS-Fast 是否正常運行。`);
    }
  } catch (err) {
    console.error('[DPLS-Fast] Cannot connect to server:', err);
    showNotification('⚠️ 無法連線至 DPLS-Fast', '請確認 DPLS-Fast 桌面應用（./dpls-gui）已啟動！');
  }
}

function showNotification(title, message) {
  chrome.notifications.create({
    type: 'basic',
    iconUrl: 'icons/icon128.png',
    title,
    message,
    priority: 2
  });
}

// Heartbeat badge update
async function checkServerStatus() {
  const config = await chrome.storage.local.get(DEFAULT_CONFIG);
  try {
    const res = await fetch(`${config.serverUrl}/api/tasks`, { cache: 'no-store' });
    if (res.ok) {
      const tasks = await res.json();
      const active = tasks.filter(t => t.state === 'Downloading').length;
      if (active > 0) {
        chrome.action.setBadgeText({ text: `${active}` });
        chrome.action.setBadgeBackgroundColor({ color: '#2ea043' });
      } else {
        chrome.action.setBadgeText({ text: 'ON' });
        chrome.action.setBadgeBackgroundColor({ color: '#58a6ff' });
      }
      return;
    }
  } catch (e) {}

  chrome.action.setBadgeText({ text: 'OFF' });
  chrome.action.setBadgeBackgroundColor({ color: '#f85149' });
}

setInterval(checkServerStatus, 3000);
checkServerStatus();
