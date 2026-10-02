document.addEventListener('DOMContentLoaded', async () => {
  // Ensure the window immediately grabs focus and stays on top
  try {
    window.focus();
  } catch (e) {}

  const params = new URLSearchParams(window.location.search);
  const targetUrl = params.get('url') || '';
  const initialFilename = params.get('filename') || '';
  const referer = params.get('referer') || '';

  // Setup View Elements
  const setupView = document.getElementById('setupView');
  const inputUrl = document.getElementById('inputUrl');
  const inputFilename = document.getElementById('inputFilename');
  const inputOutputDir = document.getElementById('inputOutputDir');
  const btnBrowseDir = document.getElementById('btnBrowseDir');
  const selectConnections = document.getElementById('selectConnections');
  const selectSpeedLimit = document.getElementById('selectSpeedLimit');
  const selectCategory = document.getElementById('selectCategory');
  const displayFileSize = document.getElementById('displayFileSize');
  const displayRangeSupport = document.getElementById('displayRangeSupport');
  const probeBanner = document.getElementById('probeBanner');
  const probeSpinner = document.getElementById('probeSpinner');
  const probeStatusText = document.getElementById('probeStatusText');
  const tagThreads = document.getElementById('tagThreads');
  const btnCopyUrl = document.getElementById('btnCopyUrl');
  const btnCancel = document.getElementById('btnCancel');
  const btnDownloadLater = document.getElementById('btnDownloadLater');
  const btnDownloadNow = document.getElementById('btnDownloadNow');
  const selectLanguage = document.getElementById('selectLanguage');

  // Mode Switcher Elements
  const btnModeAuto = document.getElementById('btnModeAuto');
  const btnModeManual = document.getElementById('btnModeManual');
  const autoTuneCard = document.getElementById('autoTuneCard');
  const autoTuneStatusChip = document.getElementById('autoTuneStatusChip');
  const autoTuneThreadsVal = document.getElementById('autoTuneThreadsVal');
  const autoTuneCategoryVal = document.getElementById('autoTuneCategoryVal');
  const autoTuneResilienceVal = document.getElementById('autoTuneResilienceVal');
  const manualControlsGroup = document.getElementById('manualControlsGroup');

  // Advanced Controls Elements
  const btnToggleAdvanced = document.getElementById('btnToggleAdvanced');
  const advancedContent = document.getElementById('advancedContent');
  const advancedArrow = document.getElementById('advancedArrow');
  const advBadge = document.getElementById('advBadge');
  const inputReferer = document.getElementById('inputReferer');
  const selectUserAgent = document.getElementById('selectUserAgent');
  const inputCookies = document.getElementById('inputCookies');
  const selectTimeout = document.getElementById('selectTimeout');
  const selectRetries = document.getElementById('selectRetries');
  const selectMinChunk = document.getElementById('selectMinChunk');
  const selectCollisionPolicy = document.getElementById('selectCollisionPolicy');
  const inputExpectedHash = document.getElementById('inputExpectedHash');
  const chkPreserveTime = document.getElementById('chkPreserveTime');
  const chkAutoOpenFile = document.getElementById('chkAutoOpenFile');
  const chkAutoOpenDir = document.getElementById('chkAutoOpenDir');
  const chkAutoClose = document.getElementById('chkAutoClose');
  const chkRemember = document.getElementById('chkRemember');

  // Progress View Elements
  const progressView = document.getElementById('progressView');
  const selectLanguageProgress = document.getElementById('selectLanguageProgress');
  const dlTargetFilename = document.getElementById('dlTargetFilename');
  const dlTargetPath = document.getElementById('dlTargetPath');
  const dlStatusBadge = document.getElementById('dlStatusBadge');
  const dlStatusDetail = document.getElementById('dlStatusDetail');
  const dlPercent = document.getElementById('dlPercent');
  const progressBarFill = document.getElementById('progressBarFill');
  const dlBytesText = document.getElementById('dlBytesText');
  const dlConnectionsText = document.getElementById('dlConnectionsText');
  const dlSpeed = document.getElementById('dlSpeed');
  const dlEta = document.getElementById('dlEta');
  const dlRangeStatus = document.getElementById('dlRangeStatus');
  const dlSegmentCount = document.getElementById('dlSegmentCount');
  const dlSegmentBar = document.getElementById('dlSegmentBar');
  const dlHashBanner = document.getElementById('dlHashBanner');
  const dlHashValue = document.getElementById('dlHashValue');
  const dlHashMatchStatus = document.getElementById('dlHashMatchStatus');
  const btnCopyHash = document.getElementById('btnCopyHash');
  const activeActions = document.getElementById('activeActions');
  const completedActions = document.getElementById('completedActions');
  const btnDlMinimize = document.getElementById('btnDlMinimize');
  const btnDlCancel = document.getElementById('btnDlCancel');
  const btnDlPauseResume = document.getElementById('btnDlPauseResume');
  const btnDlClose = document.getElementById('btnDlClose');
  const btnDlOpenDir = document.getElementById('btnDlOpenDir');
  const btnDlOpenFile = document.getElementById('btnDlOpenFile');

  inputUrl.value = targetUrl;
  if (initialFilename) {
    inputFilename.value = initialFilename;
  }
  if (referer && inputReferer) {
    inputReferer.value = referer;
  }

  // Auto-fetch Session Cookies from browser for targetUrl
  if (typeof chrome !== 'undefined' && chrome.cookies && targetUrl) {
    try {
      chrome.cookies.getAll({ url: targetUrl }, (cookies) => {
        if (cookies && cookies.length > 0) {
          const cookieStr = cookies.map(c => `${c.name}=${c.value}`).join('; ');
          if (inputCookies && !inputCookies.value) {
            inputCookies.value = cookieStr;
          }
        }
      });
    } catch (e) {}
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

    document.querySelectorAll('.theme-dot-preview').forEach(dot => {
      dot.style.background = hexColor;
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

  // Hook Theme Popovers in dialog
  document.querySelectorAll('.btn-theme-toggle').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const popover = btn.parentElement.querySelector('.theme-menu-popover');
      if (popover) {
        const isOpen = popover.classList.contains('is-open');
        document.querySelectorAll('.theme-menu-popover').forEach(p => p.classList.remove('is-open'));
        if (!isOpen) popover.classList.add('is-open');
      }
    });
  });

  document.querySelectorAll('.theme-swatch-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const color = btn.dataset.color;
      const title = btn.getAttribute('title') || '';
      applyTheme(color, title);
      document.querySelectorAll('.theme-menu-popover').forEach(p => p.classList.remove('is-open'));
    });
  });

  window.addEventListener('click', () => {
    document.querySelectorAll('.theme-menu-popover').forEach(p => p.classList.remove('is-open'));
  });

  // Language switcher setup
  const langConfig = await chrome.storage.local.get(['dpls_language']);
  let activeLang = langConfig.dpls_language || detectInitialLanguage();
  if (selectLanguage) selectLanguage.value = activeLang;
  if (selectLanguageProgress) selectLanguageProgress.value = activeLang;
  setLanguage(activeLang);
  if (window.initCustomSelects) {
    window.initCustomSelects();
  }

  function handleLanguageChange(newLang) {
    activeLang = newLang;
    if (selectLanguage) selectLanguage.value = newLang;
    if (selectLanguageProgress) selectLanguageProgress.value = newLang;
    setLanguage(newLang);
    document.querySelectorAll('select').forEach(sel => {
      if (sel._syncCustomSelect) sel._syncCustomSelect();
    });
    updateThreadTag(selectConnections.value);
    if (lastProbeData) {
      updateAutoTuneCard(lastProbeData.total_size, lastProbeData.supports_range);
    }
    if (lastTaskSnapshot) {
      renderTaskProgress(lastTaskSnapshot);
    }
  }

  if (selectLanguage) {
    selectLanguage.addEventListener('change', () => handleLanguageChange(selectLanguage.value));
  }
  if (selectLanguageProgress) {
    selectLanguageProgress.addEventListener('change', () => handleLanguageChange(selectLanguageProgress.value));
  }

  // Load saved preferences
  const config = await chrome.storage.local.get({
    connections: 64,
    speedLimit: '0',
    collisionPolicy: 'auto_rename',
    autoOpenFile: false,
    autoOpenDir: false,
    autoClose: false,
    serverUrl: 'http://127.0.0.1:6800',
    lastOutputDir: ''
  });

  if (config.connections) {
    selectConnections.value = String(config.connections);
    updateThreadTag(config.connections);
  }
  if (config.speedLimit) selectSpeedLimit.value = config.speedLimit;
  if (config.collisionPolicy) selectCollisionPolicy.value = config.collisionPolicy;
  chkAutoOpenFile.checked = !!config.autoOpenFile;
  chkAutoOpenDir.checked = !!config.autoOpenDir;
  chkAutoClose.checked = !!config.autoClose;

  if (config.lastOutputDir) {
    inputOutputDir.value = config.lastOutputDir;
  }

  // Auto-detect category from initial filename / url
  detectCategory(initialFilename || targetUrl);

  // Mode Switcher State & Handlers
  let currentMode = 'auto'; // 'auto' | 'manual'
  let autoCalculatedThreads = 64;
  let lastProbeData = null;

  function updateAutoTuneCard(totalSize, supportsRange) {
    if (supportsRange === false) {
      autoCalculatedThreads = 1;
      autoTuneThreadsVal.textContent = '1 (單線程保護模式)';
      autoTuneThreadsVal.style.color = '#fbbf24';
    } else if (totalSize && totalSize < 5 * 1024 * 1024) {
      autoCalculatedThreads = 4;
      autoTuneThreadsVal.textContent = '4 併發連線 (低負載快傳)';
      autoTuneThreadsVal.style.color = '#38bdf8';
    } else if (totalSize && totalSize < 50 * 1024 * 1024) {
      autoCalculatedThreads = 16;
      autoTuneThreadsVal.textContent = '16 併發連線 (標準多線程)';
      autoTuneThreadsVal.style.color = '#38bdf8';
    } else if (totalSize && totalSize < 500 * 1024 * 1024) {
      autoCalculatedThreads = 32;
      autoTuneThreadsVal.textContent = '32 併發連線 (高速並行)';
      autoTuneThreadsVal.style.color = '#34d399';
    } else {
      autoCalculatedThreads = 64;
      autoTuneThreadsVal.textContent = '64 併發連線 (極速多路滿載)';
      autoTuneThreadsVal.style.color = '#34d399';
    }

    if (selectCategory && selectCategory.selectedIndex >= 0) {
      autoTuneCategoryVal.textContent = selectCategory.options[selectCategory.selectedIndex].text;
    }

    if (currentMode === 'auto') {
      selectConnections.value = String(autoCalculatedThreads);
      updateThreadTag(autoCalculatedThreads);
    }
  }

  if (btnModeAuto && btnModeManual) {
    btnModeAuto.addEventListener('click', () => {
      currentMode = 'auto';
      btnModeAuto.classList.add('active');
      btnModeManual.classList.remove('active');
      if (autoTuneCard) autoTuneCard.style.display = 'flex';
      if (manualControlsGroup) manualControlsGroup.style.display = 'none';
      selectConnections.value = String(autoCalculatedThreads);
      updateThreadTag(autoCalculatedThreads);
    });

    btnModeManual.addEventListener('click', () => {
      currentMode = 'manual';
      btnModeManual.classList.add('active');
      btnModeAuto.classList.remove('active');
      if (autoTuneCard) autoTuneCard.style.display = 'none';
      if (manualControlsGroup) manualControlsGroup.style.display = 'flex';
      updateThreadTag(selectConnections.value);
    });
  }

  // Toggle Advanced Section with Smooth Scroll
  btnToggleAdvanced.addEventListener('click', () => {
    const isHidden = advancedContent.style.display === 'none';
    advancedContent.style.display = isHidden ? 'flex' : 'none';
    advancedArrow.classList.toggle('rotated', isHidden);
    if (isHidden) {
      setTimeout(() => {
        advancedContent.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      }, 50);
    }
  });

  // System directories cache
  let systemDirs = {};
  async function loadSystemDirs() {
    try {
      const res = await fetch(`${config.serverUrl}/api/system-dirs`);
      if (res.ok) {
        systemDirs = await res.json();
      }
    } catch (e) {}
  }
  loadSystemDirs();

  // Quick Path Chips
  document.querySelectorAll('.path-chip').forEach(chip => {
    chip.addEventListener('click', () => {
      const type = chip.getAttribute('data-type');
      if (systemDirs[type]) {
        inputOutputDir.value = systemDirs[type];
        document.querySelectorAll('.path-chip').forEach(c => c.classList.remove('active'));
        chip.classList.add('active');
      }
    });
  });

  // Native Directory Picker Browse button
  const btnBrowseIcon = document.getElementById('btnBrowseIcon');
  const btnBrowseText = document.getElementById('btnBrowseText');

  btnBrowseDir.addEventListener('click', async () => {
    if (btnBrowseIcon) btnBrowseIcon.textContent = '⏳';
    if (btnBrowseText) btnBrowseText.textContent = t('btn_browsing');
    try {
      const res = await fetch(`${config.serverUrl}/api/dialog/pick-dir`, { method: 'POST' });
      if (res.ok) {
        const data = await res.json();
        if (data.path && !data.canceled) {
          inputOutputDir.value = data.path;
          document.querySelectorAll('.path-chip').forEach(c => c.classList.remove('active'));
        }
      }
    } catch (e) {
      alert(t('browse_error') || '無法呼叫系統資料夾選擇視窗，請確認後端服務運行中。');
    } finally {
      if (btnBrowseIcon) btnBrowseIcon.textContent = '📂';
      if (btnBrowseText) btnBrowseText.textContent = t('btn_browse');
    }
  });

  // Copy URL button
  btnCopyUrl.addEventListener('click', () => {
    navigator.clipboard.writeText(targetUrl);
    btnCopyUrl.textContent = '✅';
    setTimeout(() => { btnCopyUrl.textContent = '📋'; }, 1500);
  });

  // Copy Hash button
  btnCopyHash.addEventListener('click', () => {
    if (dlHashValue.textContent) {
      navigator.clipboard.writeText(dlHashValue.textContent);
      btnCopyHash.textContent = '✅';
      setTimeout(() => { btnCopyHash.textContent = '📋'; }, 1500);
    }
  });

  // Threads dropdown change
  selectConnections.addEventListener('change', () => {
    updateThreadTag(selectConnections.value);
  });

  // Category change logic
  let baseDownloadDir = config.lastOutputDir || '';
  selectCategory.addEventListener('change', () => {
    const cat = selectCategory.value;
    if (baseDownloadDir && cat !== 'general') {
      const sep = baseDownloadDir.includes('\\') ? '\\' : '/';
      const catFolder = getCategoryFolderName(cat);
      const parent = baseDownloadDir.replace(/[\\/](Compressed|Video|Audio|Programs|Documents)$/i, '');
      inputOutputDir.value = `${parent}${sep}${catFolder}`;
    }
    if (autoTuneCategoryVal && selectCategory.selectedIndex >= 0) {
      autoTuneCategoryVal.textContent = selectCategory.options[selectCategory.selectedIndex].text;
    }
  });

  function updateThreadTag(val) {
    tagThreads.textContent = `${val}${t('tag_threads_suffix')}`;
    if (val >= 64) {
      tagThreads.style.borderColor = 'rgba(16, 185, 129, 0.6)';
      tagThreads.style.color = '#34d399';
    } else if (val >= 32) {
      tagThreads.style.borderColor = 'rgba(56, 189, 248, 0.6)';
      tagThreads.style.color = '#38bdf8';
    } else {
      tagThreads.style.borderColor = 'rgba(255, 255, 255, 0.15)';
      tagThreads.style.color = '#94a3b8';
    }
  }

  function detectCategory(nameOrUrl) {
    const extMatch = nameOrUrl.toLowerCase().match(/\.([a-z0-9]+)(?:[\?#]|$)/);
    if (!extMatch) return;
    const ext = extMatch[1];
    if (['zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'iso'].includes(ext)) {
      selectCategory.value = 'compressed';
    } else if (['mp4', 'mkv', 'avi', 'mov', 'flv', 'wmv', 'webm'].includes(ext)) {
      selectCategory.value = 'video';
    } else if (['mp3', 'flac', 'wav', 'aac', 'ogg'].includes(ext)) {
      selectCategory.value = 'audio';
    } else if (['exe', 'msi', 'deb', 'rpm', 'apk', 'pkg', 'dmg'].includes(ext)) {
      selectCategory.value = 'programs';
    } else if (['pdf', 'epub', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'txt'].includes(ext)) {
      selectCategory.value = 'documents';
    }

    if (autoTuneCategoryVal && selectCategory.selectedIndex >= 0) {
      autoTuneCategoryVal.textContent = selectCategory.options[selectCategory.selectedIndex].text;
    }
  }

  function getCategoryFolderName(cat) {
    switch (cat) {
      case 'compressed': return 'Compressed';
      case 'video': return 'Video';
      case 'audio': return 'Audio';
      case 'programs': return 'Programs';
      case 'documents': return 'Documents';
      default: return '';
    }
  }

  // Probe target
  async function probeTarget() {
    if (!targetUrl) return;
    try {
      const probeUrl = `${config.serverUrl}/api/probe?url=${encodeURIComponent(targetUrl)}`;
      const res = await fetch(probeUrl);
      if (res.ok) {
        const data = await res.json();
        lastProbeData = data;

        if (data.filename && (!inputFilename.value || inputFilename.value === 'download.bin')) {
          inputFilename.value = data.filename;
          detectCategory(data.filename);
        }

        if (data.total_size) {
          displayFileSize.textContent = formatBytes(data.total_size);
        } else {
          displayFileSize.textContent = t('unknown_size');
        }

        if (data.supports_range) {
          displayRangeSupport.innerHTML = `<span style="color: #34d399;">${t('range_yes')}</span>`;
        } else {
          displayRangeSupport.innerHTML = `<span style="color: #fbbf24;">${t('range_no')}</span>`;
        }

        if (!inputOutputDir.value && data.default_output_dir) {
          baseDownloadDir = data.default_output_dir;
          inputOutputDir.value = data.default_output_dir;
        } else if (data.default_output_dir && !baseDownloadDir) {
          baseDownloadDir = data.default_output_dir;
        }

        // Apply Smart Auto-Tune
        updateAutoTuneCard(data.total_size, data.supports_range);

        probeSpinner.style.display = 'none';
        probeBanner.classList.add('success');
        probeStatusText.textContent = t('probe_ready');
        window.focus();
        return;
      }
    } catch (e) {
      console.warn('[DPLS-Fast Dialog] Probe warning:', e);
    }

    probeSpinner.style.display = 'none';
    probeStatusText.textContent = t('probe_fallback');
    if (!inputFilename.value) {
      inputFilename.value = extractBasenameFromUrl(targetUrl);
    }
    displayFileSize.textContent = t('unknown_size');
    displayRangeSupport.textContent = t('range_probing');
    updateAutoTuneCard(null, true);
    window.focus();
  }

  probeTarget();

  function extractBasenameFromUrl(url) {
    try {
      const parsed = new URL(url);
      const name = parsed.pathname.split('/').filter(Boolean).pop();
      return name ? decodeURIComponent(name) : 'download.bin';
    } catch (e) {
      return 'download.bin';
    }
  }

  function formatBytes(bytes) {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function formatEta(sec) {
    if (sec === null || sec === undefined) return '--:--';
    if (sec <= 0) return t('eta_done');
    const h = Math.floor(sec / 3600);
    const m = Math.floor((sec % 3600) / 60);
    const s = sec % 60;
    if (h > 0) {
      return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
    }
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }

  btnCancel.addEventListener('click', () => {
    window.close();
  });

  // Keyboard shortcut: Escape to close, Enter to start
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      window.close();
    } else if (e.key === 'Enter' && setupView.style.display !== 'none' && !btnDownloadNow.disabled) {
      btnDownloadNow.click();
    }
  });

  function getFinalRequestPayload(startImmediately) {
    const filename = inputFilename.value.trim() || null;
    const outputDir = inputOutputDir.value.trim() || null;
    const connections = currentMode === 'auto'
      ? autoCalculatedThreads
      : (parseInt(selectConnections.value, 10) || 64);
    const speedLimitBps = parseInt(selectSpeedLimit.value, 10) || 0;
    const collisionPolicy = selectCollisionPolicy ? selectCollisionPolicy.value : 'auto_rename';
    const expectedHash = (inputExpectedHash && inputExpectedHash.value.trim().toLowerCase()) || null;

    // Advanced HTTP Headers
    const userAgentChoice = selectUserAgent ? selectUserAgent.value : 'default';
    let finalUserAgent = null;
    if (userAgentChoice === 'chrome') {
      finalUserAgent = navigator.userAgent;
    } else if (userAgentChoice === 'idm') {
      finalUserAgent = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) DownloadManager/IDM';
    } else if (userAgentChoice === 'curl') {
      finalUserAgent = 'curl/8.5.0';
    }

    const finalReferer = (inputReferer && inputReferer.value.trim()) || referer || null;
    const finalCookies = (inputCookies && inputCookies.value.trim()) || null;

    return {
      url: targetUrl,
      filename,
      output_dir: outputDir,
      connections,
      start_immediately: startImmediately,
      speed_limit_bps: speedLimitBps > 0 ? speedLimitBps : null,
      collision_policy: collisionPolicy,
      expected_hash: expectedHash,
      user_agent: finalUserAgent,
      referer: finalReferer,
      cookies: finalCookies
    };
  }

  // "排入待傳佇列" (稍後下載)
  btnDownloadLater.addEventListener('click', async () => {
    const payload = getFinalRequestPayload(false);

    if (chkRemember.checked) {
      chrome.storage.local.set({
        connections: payload.connections,
        speedLimit: selectSpeedLimit.value,
        collisionPolicy: payload.collision_policy,
        autoOpenFile: chkAutoOpenFile.checked,
        autoOpenDir: chkAutoOpenDir.checked,
        autoClose: chkAutoClose.checked,
        lastOutputDir: payload.output_dir || baseDownloadDir
      });
    }

    btnDownloadLater.disabled = true;
    btnDownloadNow.disabled = true;

    try {
      const res = await fetch(`${config.serverUrl}/api/add`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });

      if (res.ok) {
        chrome.notifications.create({
          type: 'basic',
          iconUrl: 'icons/icon128.png',
          title: '⏳ DPLS-Fast: ' + t('btn_queue'),
          message: `${payload.filename || targetUrl}`,
          priority: 2
        });
        window.close();
      } else {
        alert('加入待傳佇列失敗，伺服器錯誤代碼: ' + res.status);
        btnDownloadLater.disabled = false;
        btnDownloadNow.disabled = false;
      }
    } catch (e) {
      alert('無法連線到 DPLS-Fast 服務，請確認桌面端已在背景運行！');
      btnDownloadLater.disabled = false;
      btnDownloadNow.disabled = false;
    }
  });

  // "開始傳輸" -> In-Window Progress View
  let currentTaskId = null;
  let currentTaskState = 'Starting';
  let ws = null;
  let pollInterval = null;
  let expectedHashValue = null;
  let lastTaskSnapshot = null;

  btnDownloadNow.addEventListener('click', async () => {
    const payload = getFinalRequestPayload(true);
    expectedHashValue = payload.expected_hash;

    if (chkRemember.checked) {
      chrome.storage.local.set({
        connections: payload.connections,
        speedLimit: selectSpeedLimit.value,
        collisionPolicy: payload.collision_policy,
        autoOpenFile: chkAutoOpenFile.checked,
        autoOpenDir: chkAutoOpenDir.checked,
        autoClose: chkAutoClose.checked,
        lastOutputDir: payload.output_dir || baseDownloadDir
      });
    }

    btnDownloadNow.disabled = true;
    btnDownloadNow.textContent = t('btn_starting');

    try {
      const res = await fetch(`${config.serverUrl}/api/add`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });

      if (!res.ok) {
        alert('建立傳輸任務失敗，伺服器回應狀態碼：' + res.status);
        btnDownloadNow.disabled = false;
        btnDownloadNow.textContent = t('btn_start');
        return;
      }

      const data = await res.json();
      currentTaskId = data.id;

      // SWITCH TO PROGRESS VIEW!
      setupView.style.display = 'none';
      progressView.style.display = 'flex';
      window.focus();

      dlTargetFilename.textContent = payload.filename || extractBasenameFromUrl(targetUrl);
      dlTargetPath.textContent = payload.output_dir || baseDownloadDir;
      dlConnectionsText.textContent = `${payload.connections} ${t('connections_full')}`;

      startLiveTracking(currentTaskId);

    } catch (err) {
      alert('無法連線到 DPLS-Fast 伺服器，請確認桌面端已在背景運行！');
      btnDownloadNow.disabled = false;
      btnDownloadNow.textContent = t('btn_start');
    }
  });

  function startLiveTracking(taskId) {
    const wsUrl = config.serverUrl.replace(/^http/, 'ws') + '/ws';
    try {
      ws = new WebSocket(wsUrl);
      ws.onmessage = (event) => {
        try {
          const snap = JSON.parse(event.data);
          if (snap.id === taskId) {
            renderTaskProgress(snap);
          }
        } catch (e) {}
      };
      ws.onerror = () => { startPolling(taskId); };
      ws.onclose = () => { startPolling(taskId); };
    } catch (e) {
      startPolling(taskId);
    }

    pollSingleTask(taskId);
  }

  function startPolling(taskId) {
    if (pollInterval) return;
    pollInterval = setInterval(() => {
      pollSingleTask(taskId);
    }, 400);
  }

  async function pollSingleTask(taskId) {
    try {
      const res = await fetch(`${config.serverUrl}/api/task/${taskId}`);
      if (res.ok) {
        const snap = await res.json();
        renderTaskProgress(snap);
      }
    } catch (e) {}
  }

  function renderTaskProgress(task) {
    lastTaskSnapshot = task;
    currentTaskState = task.state;
    dlTargetFilename.textContent = task.filename || t('dl_preparing');
    dlTargetPath.textContent = task.output_path || '';

    const pct = (task.progress_ratio * 100).toFixed(1);
    dlPercent.textContent = `${pct}%`;
    progressBarFill.style.width = `${pct}%`;

    const downloadedStr = formatBytes(task.downloaded_bytes);
    const totalStr = task.total_size ? formatBytes(task.total_size) : t('unknown_size');
    dlBytesText.textContent = `${downloadedStr} / ${totalStr}`;

    const speedMB = (task.speed_bps / (1024 * 1024)).toFixed(2);
    dlSpeed.textContent = `${speedMB} MB/s`;
    dlEta.textContent = formatEta(task.eta_seconds);
    dlConnectionsText.textContent = `${task.connections} ${t('connections_full')}`;
    dlRangeStatus.textContent = task.supports_range ? t('mode_range') : t('mode_stream');

    // Render IDM Segment Visualizer Bar
    if (task.segments && task.segments.length > 0 && task.total_size) {
      dlSegmentCount.textContent = `${t('segment_count_prefix')}${task.segments.length}`;
      let slicesHtml = '';
      const total = task.total_size;

      task.segments.forEach(seg => {
        const leftPct = ((seg.start_offset / total) * 100).toFixed(2);
        const segLen = seg.end_offset - seg.start_offset + 1;
        const widthPct = Math.max(0.2, (segLen / total) * 100).toFixed(2);
        const downloadedInSeg = seg.downloaded_bytes || (seg.status === 'Completed' ? segLen : 0);
        const fillRatio = Math.min(100, (downloadedInSeg / segLen) * 100).toFixed(1);

        const isActive = seg.status === 'Downloading' || (seg.status === 'Pending' && !task.is_paused);
        const isCompleted = seg.status === 'Completed';
        const sliceClass = isCompleted ? 'completed' : (isActive ? 'active' : '');

        slicesHtml += `
          <div class="segment-slice ${sliceClass}" style="left: ${leftPct}%; width: ${widthPct}%;">
            <div class="segment-slice-fill" style="width: ${fillRatio}%;"></div>
          </div>
        `;
      });
      dlSegmentBar.innerHTML = slicesHtml;
    } else {
      if (task.state === 'Completed') {
        dlSegmentBar.innerHTML = '<div class="segment-slice completed" style="left:0; width:100%;"><div class="segment-slice-fill" style="width:100%;"></div></div>';
      } else {
        dlSegmentBar.innerHTML = `<div class="segment-placeholder">${t('segment_slicing')}</div>`;
      }
    }

    // State handling
    if (task.state === 'Downloading') {
      dlStatusBadge.textContent = `⚡ ${t('dl_status_active')} (${task.connections})`;
      dlStatusBadge.style.color = '#34d399';
      dlStatusBadge.style.borderColor = 'rgba(16, 185, 129, 0.6)';
      dlStatusDetail.textContent = t('dl_detail_active');
      btnDlPauseResume.textContent = t('btn_pause');
      btnDlPauseResume.className = 'btn btn-later';
    } else if (task.state === 'Paused') {
      dlStatusBadge.textContent = `⏸ ${t('dl_status_paused')}`;
      dlStatusBadge.style.color = '#fbbf24';
      dlStatusBadge.style.borderColor = 'rgba(245, 158, 11, 0.6)';
      dlStatusDetail.textContent = t('dl_detail_paused');
      btnDlPauseResume.textContent = t('btn_resume');
      btnDlPauseResume.className = 'btn btn-primary';
      dlSpeed.textContent = '0.0 MB/s';
    } else if (task.state === 'Completed') {
      dlStatusBadge.textContent = `✅ ${t('dl_status_completed')}`;
      dlStatusBadge.style.color = '#34d399';
      dlStatusBadge.style.borderColor = '#34d399';
      dlStatusDetail.textContent = t('dl_detail_completed');
      dlPercent.textContent = '100%';
      progressBarFill.style.width = '100%';
      progressBarFill.style.animation = 'none';
      progressBarFill.style.background = '#10b981';
      dlSpeed.textContent = 'OK';
      dlEta.textContent = '00:00';

      activeActions.style.display = 'none';
      completedActions.style.display = 'flex';

      if (pollInterval) {
        clearInterval(pollInterval);
        pollInterval = null;
      }

      // Fetch SHA-256 Hash
      fetchFileHash(task.id);

      // Automated post-transfer actions
      handlePostTransferActions(task.id);
    } else if (typeof task.state === 'object' && task.state.Error) {
      dlStatusBadge.textContent = `❌ ${t('dl_status_failed')}`;
      dlStatusBadge.style.color = '#f43f5e';
      dlStatusBadge.style.borderColor = '#f43f5e';
      dlStatusDetail.textContent = `${t('dl_status_failed')}: ${task.state.Error}`;
      btnDlPauseResume.textContent = t('btn_retry');
    }
  }

  let hashFetched = false;
  async function fetchFileHash(taskId) {
    if (hashFetched) return;
    hashFetched = true;
    dlHashBanner.style.display = 'flex';
    try {
      const res = await fetch(`${config.serverUrl}/api/task/${taskId}/hash`);
      if (res.ok) {
        const data = await res.json();
        dlHashValue.textContent = data.sha256;

        if (expectedHashValue) {
          if (data.sha256.toLowerCase() === expectedHashValue.toLowerCase()) {
            dlHashMatchStatus.innerHTML = `<span style="color: #34d399; font-weight: 600;">${t('hash_match')}</span>`;
          } else {
            dlHashMatchStatus.innerHTML = `<span style="color: #f43f5e; font-weight: 600;">${t('hash_mismatch')}</span>`;
          }
        }
      }
    } catch (e) {
      dlHashValue.textContent = 'Error computing hash';
    }
  }

  let postActionsTriggered = false;
  async function handlePostTransferActions(taskId) {
    if (postActionsTriggered) return;
    postActionsTriggered = true;

    if (chkAutoOpenFile.checked) {
      fetch(`${config.serverUrl}/api/open/${taskId}`, { method: 'POST' });
    }
    if (chkAutoOpenDir.checked) {
      fetch(`${config.serverUrl}/api/open-dir/${taskId}`, { method: 'POST' });
    }
    if (chkAutoClose.checked) {
      setTimeout(() => {
        window.close();
      }, 1500);
    }
  }

  // Live Control Buttons
  btnDlPauseResume.addEventListener('click', async () => {
    if (!currentTaskId) return;
    if (currentTaskState === 'Downloading') {
      await fetch(`${config.serverUrl}/api/pause/${currentTaskId}`, { method: 'POST' });
    } else {
      await fetch(`${config.serverUrl}/api/resume/${currentTaskId}`, { method: 'POST' });
    }
    pollSingleTask(currentTaskId);
  });

  btnDlCancel.addEventListener('click', async () => {
    if (!currentTaskId) return;
    if (confirm(t('confirm_abort'))) {
      await fetch(`${config.serverUrl}/api/task/${currentTaskId}`, { method: 'DELETE' });
      window.close();
    }
  });

  btnDlMinimize.addEventListener('click', () => {
    window.close();
  });

  btnDlClose.addEventListener('click', () => {
    window.close();
  });

  btnDlOpenFile.addEventListener('click', async () => {
    if (!currentTaskId) return;
    await fetch(`${config.serverUrl}/api/open/${currentTaskId}`, { method: 'POST' });
  });

  btnDlOpenDir.addEventListener('click', async () => {
    if (!currentTaskId) return;
    await fetch(`${config.serverUrl}/api/open-dir/${currentTaskId}`, { method: 'POST' });
  });
});
