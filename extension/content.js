// DPLS-Fast Content Script: IDM-like click interceptor

const DEFAULT_EXTENSIONS = new Set([
  'zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'iso', 'img', 'dmg',
  'exe', 'msi', 'deb', 'rpm', 'apk', 'pkg',
  'mp4', 'mkv', 'avi', 'mov', 'flv', 'wmv', 'webm',
  'mp3', 'flac', 'wav', 'aac', 'ogg',
  'pdf', 'epub', 'bin', 'torrent'
]);

let config = {
  autoIntercept: true,
  interceptAll: false,
  extensions: Array.from(DEFAULT_EXTENSIONS)
};

// Sync config from extension storage
function loadConfig() {
  chrome.storage.local.get({
    autoIntercept: true,
    interceptAll: false,
    extensions: Array.from(DEFAULT_EXTENSIONS)
  }, (stored) => {
    config = stored;
  });
}

loadConfig();
chrome.storage.onChanged.addListener(loadConfig);

function getFileExtension(url) {
  try {
    const u = new URL(url, window.location.href);
    const pathname = u.pathname;
    const dot = pathname.lastIndexOf('.');
    if (dot !== -1) {
      return pathname.slice(dot + 1).toLowerCase().split('/')[0];
    }
  } catch (e) {}
  return '';
}

// Intercept link clicks in capture phase
document.addEventListener('click', (e) => {
  if (!config.autoIntercept) return;

  // IDM Standard: Holding Alt key bypasses interception
  if (e.altKey) return;

  const a = e.target.closest('a');
  if (!a || !a.href) return;

  const href = a.href;
  if (href.startsWith('javascript:') || href.startsWith('#') || href.startsWith('mailto:') || href.startsWith('tel:')) {
    return;
  }

  const ext = getFileExtension(href);
  const hasDownloadAttr = a.hasAttribute('download');
  const isMatch = config.interceptAll || hasDownloadAttr || (ext && config.extensions.includes(ext));

  if (isMatch) {
    console.log('[DPLS-Fast] Intercepting link click:', href);
    e.preventDefault();
    e.stopPropagation();

    chrome.runtime.sendMessage({
      type: 'LINK_CLICKED',
      url: href,
      filename: a.getAttribute('download') || null,
      referer: window.location.href
    });
  }
}, true);
