// ========================================================
// Liquid Glass Custom Select Enhancer (自製毛玻璃自定義下拉選單)
// ========================================================

function closeAllCustomSelects() {
  document.querySelectorAll('.dpls-select-trigger.is-open').forEach(el => el.classList.remove('is-open'));
  document.querySelectorAll('.dpls-select-menu.is-open').forEach(el => el.classList.remove('is-open'));
}

function enhanceSelect(selectEl, isInline = false) {
  if (!selectEl || selectEl._customEnhanced) return;
  selectEl._customEnhanced = true;

  selectEl.classList.add('dpls-hidden-select');

  const wrapper = document.createElement('div');
  wrapper.className = 'dpls-select-wrapper' + (isInline ? ' inline-select' : '');

  const trigger = document.createElement('div');
  const inheritedClass = selectEl.className.replace('dpls-hidden-select', '').trim();
  trigger.className = 'dpls-select-trigger ' + (inheritedClass || 'form-control');
  trigger.setAttribute('tabindex', '0');

  const textSpan = document.createElement('span');
  textSpan.className = 'dpls-select-val';

  const arrowSpan = document.createElement('span');
  arrowSpan.className = 'dpls-select-arrow';
  arrowSpan.innerHTML = `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>`;

  trigger.appendChild(textSpan);
  trigger.appendChild(arrowSpan);

  selectEl.parentNode.insertBefore(wrapper, selectEl);
  wrapper.appendChild(selectEl);
  wrapper.appendChild(trigger);

  const menu = document.createElement('div');
  menu.className = 'dpls-select-menu';
  document.body.appendChild(menu);

  function syncText() {
    const selOpt = selectEl.options[selectEl.selectedIndex] || selectEl.options[0];
    textSpan.textContent = selOpt ? selOpt.text : '';
  }

  function renderOptions() {
    syncText();
    menu.innerHTML = '';
    Array.from(selectEl.options).forEach((opt, idx) => {
      const optDiv = document.createElement('div');
      optDiv.className = 'dpls-select-option' + (opt.value === selectEl.value ? ' is-selected' : '');
      optDiv.dataset.value = opt.value;
      optDiv.dataset.index = idx;

      const label = document.createElement('span');
      label.textContent = opt.text;

      const check = document.createElement('span');
      check.className = 'dpls-select-check';
      check.textContent = '✓';

      optDiv.appendChild(label);
      optDiv.appendChild(check);

      optDiv.addEventListener('click', (e) => {
        e.stopPropagation();
        if (selectEl.value !== opt.value) {
          selectEl.value = opt.value;
          syncText();
          selectEl.dispatchEvent(new Event('change', { bubbles: true }));
          if (typeof selectEl.onchange === 'function') {
            selectEl.onchange();
          }
        }
        closeMenu();
      });

      menu.appendChild(optDiv);
    });
  }

  function openMenu() {
    closeAllCustomSelects();
    renderOptions();

    const rect = trigger.getBoundingClientRect();
    const menuHeight = Math.min(selectEl.options.length * 36 + 14, 250);
    const spaceBelow = window.innerHeight - rect.bottom;
    const spaceAbove = rect.top;

    let top;
    if (spaceBelow < menuHeight && spaceAbove > spaceBelow) {
      top = rect.top - menuHeight - 6;
      menu.style.transformOrigin = 'bottom left';
    } else {
      top = rect.bottom + 6;
      menu.style.transformOrigin = 'top left';
    }
    if (top < 10) top = 10;

    const minW = isInline ? Math.max(rect.width, 130) : rect.width;
    menu.style.top = `${top}px`;
    menu.style.minWidth = `${minW}px`;

    const maxRight = window.innerWidth - 10;
    if (rect.left + minW > maxRight) {
      menu.style.left = 'auto';
      menu.style.right = `${window.innerWidth - rect.right}px`;
    } else {
      menu.style.left = `${Math.max(10, rect.left)}px`;
      menu.style.right = 'auto';
    }

    trigger.classList.add('is-open');
    menu.classList.add('is-open');

    const activeItem = menu.querySelector('.dpls-select-option.is-selected');
    if (activeItem) {
      activeItem.scrollIntoView({ block: 'nearest' });
    }
  }

  function closeMenu() {
    trigger.classList.remove('is-open');
    menu.classList.remove('is-open');
  }

  trigger.addEventListener('click', (e) => {
    e.stopPropagation();
    if (menu.classList.contains('is-open')) {
      closeMenu();
    } else {
      openMenu();
    }
  });

  trigger.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      trigger.click();
    } else if (e.key === 'Escape') {
      closeMenu();
    }
  });

  selectEl.addEventListener('change', syncText);

  // Hook property descriptor for .value
  const desc = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value');
  Object.defineProperty(selectEl, 'value', {
    get() {
      return desc.get.call(this);
    },
    set(v) {
      desc.set.call(this, v);
      syncText();
    }
  });

  syncText();

  selectEl._syncCustomSelect = () => {
    syncText();
    if (menu.classList.contains('is-open')) renderOptions();
  };
  selectEl._closeCustomSelect = closeMenu;
}

function initCustomSelects() {
  document.querySelectorAll('select').forEach(sel => {
    const isInline = sel.classList.contains('lang-picker') || sel.id === 'selectMaxConcurrent';
    enhanceSelect(sel, isInline);
  });
}

window.addEventListener('click', (e) => {
  if (!e.target.closest('.dpls-select-trigger') && !e.target.closest('.dpls-select-menu')) {
    closeAllCustomSelects();
  }
});
window.addEventListener('scroll', () => {
  closeAllCustomSelects();
}, true);
window.addEventListener('resize', closeAllCustomSelects);
window.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') closeAllCustomSelects();
});

window.enhanceSelect = enhanceSelect;
window.initCustomSelects = initCustomSelects;
window.closeAllCustomSelects = closeAllCustomSelects;
