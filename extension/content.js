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
  mediaSniffer: true,
  extensions: Array.from(DEFAULT_EXTENSIONS)
};

// Sync config from extension storage
function loadConfig() {
  chrome.storage.local.get({
    autoIntercept: true,
    interceptAll: false,
    mediaSniffer: true,
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

// ==========================================
// IDM-Style Web Media & Video Sniffer
// ==========================================
function injectSnifferStyles() {
  if (document.getElementById('dpls-sniffer-style')) return;
  const style = document.createElement('style');
  style.id = 'dpls-sniffer-style';
  style.textContent = `
    .dpls-sniffer-wrapper {
      position: absolute;
      z-index: 2147483640;
      pointer-events: auto;
      transition: opacity 0.25s ease, transform 0.25s ease;
    }
    .dpls-sniffer-btn {
      display: inline-flex !important;
      align-items: center !important;
      gap: 6px !important;
      background: rgba(13, 17, 23, 0.88) !important;
      backdrop-filter: blur(14px) saturate(180%) !important;
      border: 1px solid rgba(88, 166, 255, 0.45) !important;
      color: #f0f6fc !important;
      padding: 6px 12px !important;
      border-radius: 20px !important;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif !important;
      font-size: 12px !important;
      font-weight: 700 !important;
      cursor: pointer !important;
      box-shadow: 0 4px 18px rgba(0, 0, 0, 0.55), inset 0 1px 1px rgba(255, 255, 255, 0.25) !important;
      transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1) !important;
      user-select: none !important;
      text-decoration: none !important;
      line-height: 1.2 !important;
    }
    .dpls-sniffer-btn:hover {
      background: linear-gradient(135deg, #10b981, #06b6d4) !important;
      border-color: #38bdf8 !important;
      box-shadow: 0 6px 24px rgba(6, 182, 212, 0.6), inset 0 1px 1px rgba(255, 255, 255, 0.5) !important;
      transform: translateY(-2px) scale(1.05) !important;
      color: #ffffff !important;
    }
    .dpls-sniffer-icon {
      font-size: 14px !important;
    }
  `;
  document.head.appendChild(style);
}

function getCleanMediaTitle(mediaEl, defaultExt = 'mp4') {
  let title = document.title || 'video';
  title = title.replace(/[\\/:*?"<>|]/g, '_').trim();
  if (title.length > 50) {
    title = title.slice(0, 50);
  }
  return `${title}.${defaultExt}`;
}

function attachMediaSniffer(mediaEl) {
  if (!config.mediaSniffer) return;
  if (mediaEl.dataset.dplsAttached) return;

  const src = mediaEl.currentSrc || mediaEl.src || (mediaEl.querySelector('source') && mediaEl.querySelector('source').src);
  if (!src || !src.startsWith('http')) return;

  mediaEl.dataset.dplsAttached = 'true';
  injectSnifferStyles();

  const isAudio = mediaEl.tagName.toLowerCase() === 'audio';
  const label = isAudio ? '下載此音訊' : '下載此視訊';
  const defaultExt = isAudio ? 'mp3' : 'mp4';

  const wrapper = document.createElement('div');
  wrapper.className = 'dpls-sniffer-wrapper';

  const btn = document.createElement('button');
  btn.className = 'dpls-sniffer-btn';
  btn.innerHTML = `<span class="dpls-sniffer-icon">⚡</span><span>${label}</span>`;
  btn.title = `使用 DPLS-Fast 極速下載: ${src}`;

  btn.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    const currentUrl = mediaEl.currentSrc || mediaEl.src || src;
    console.log('[DPLS-Fast Sniffer] Downloading media:', currentUrl);
    chrome.runtime.sendMessage({
      type: 'LINK_CLICKED',
      url: currentUrl,
      filename: getCleanMediaTitle(mediaEl, defaultExt),
      referer: window.location.href
    });
  });

  wrapper.appendChild(btn);

  // Position wrapper relative to parent or fixed above media
  function positionBadge() {
    const rect = mediaEl.getBoundingClientRect();
    if (rect.width > 120 && rect.height > 60) {
      wrapper.style.position = 'fixed';
      wrapper.style.top = `${Math.max(10, rect.top + 10)}px`;
      wrapper.style.left = `${Math.max(10, rect.right - 130)}px`;
      wrapper.style.display = 'block';
    } else {
      wrapper.style.display = 'none';
    }
  }

  document.body.appendChild(wrapper);
  positionBadge();

  window.addEventListener('scroll', positionBadge, { passive: true });
  window.addEventListener('resize', positionBadge, { passive: true });
  mediaEl.addEventListener('play', positionBadge);
  mediaEl.addEventListener('loadeddata', positionBadge);
}

function scanMediaElements() {
  if (!config.mediaSniffer) return;
  const elements = document.querySelectorAll('video, audio');
  elements.forEach((el) => {
    attachMediaSniffer(el);
  });
}

// Observe DOM for dynamically inserted videos / streaming players
const mediaObserver = new MutationObserver(() => {
  scanMediaElements();
});

if (document.body) {
  mediaObserver.observe(document.body, { childList: true, subtree: true });
  scanMediaElements();
} else {
  document.addEventListener('DOMContentLoaded', () => {
    mediaObserver.observe(document.body, { childList: true, subtree: true });
    scanMediaElements();
  });
}

// Periodic check for delayed video sources (e.g. YouTube, Bilibili, Vimeo)
setInterval(scanMediaElements, 2500);
