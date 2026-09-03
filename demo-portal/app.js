const runButton = document.querySelector('#run-check');
const resetButton = document.querySelector('#reset-check');
const decision = document.querySelector('#decision');
const gates = Array.from(document.querySelectorAll('.gate'));
const themeToggle = document.querySelector('#theme-toggle');
const themeColor = document.querySelector('meta[name="theme-color"]');
const themePreference = window.matchMedia('(prefers-color-scheme: dark)');

const delay = (milliseconds) => new Promise((resolve) => window.setTimeout(resolve, milliseconds));

function setDecision(state, title, detail) {
  decision.className = `decision ${state}`.trim();
  decision.querySelector('.decision-icon').textContent = state === 'is-refused' ? '×' : '?';
  decision.querySelector('strong').textContent = title;
  decision.querySelector('small').textContent = detail;
}

function resetReplay() {
  gates.forEach((gate) => {
    gate.className = gate.dataset.gate === 'broker' ? 'gate broker-gate is-locked' : 'gate';
    gate.querySelector('.gate-status').textContent = gate.dataset.gate === 'broker' ? 'Locked' : 'Waiting';
  });
  setDecision('', 'Awaiting replay', 'Run the checks to see the code-backed verdict.');
  runButton.disabled = false;
  runButton.querySelector('.button-label').textContent = 'Run safety checks';
}

async function runReplay() {
  resetReplay();
  runButton.disabled = true;
  runButton.querySelector('.button-label').textContent = 'Checking';

  for (const gate of gates.slice(0, 4)) {
    gate.classList.add('is-checking');
    gate.querySelector('.gate-status').textContent = 'Checking';
    await delay(320);
    gate.classList.remove('is-checking');
    gate.classList.add('is-passed');
    gate.querySelector('.gate-status').textContent = 'Passed';
  }

  const riskGate = gates[4];
  riskGate.classList.add('is-checking');
  riskGate.querySelector('.gate-status').textContent = 'Calculating';
  await delay(520);
  riskGate.classList.remove('is-checking');
  riskGate.classList.add('is-refused');
  riskGate.querySelector('.gate-status').textContent = 'Refused';

  gates[5].classList.add('is-locked');
  gates[5].querySelector('.gate-status').textContent = 'Not reached';
  setDecision('is-refused', 'Order refused before Alpaca', '$2,525 maximum loss exceeds the $2,000 hard limit.');
  runButton.querySelector('.button-label').textContent = 'Replay complete';
  runButton.disabled = false;
}

function applyTheme(theme, persist = true) {
  document.documentElement.dataset.theme = theme;
  themeToggle.setAttribute('aria-label', `Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`);
  themeColor.setAttribute('content', theme === 'dark' ? '#0d120f' : '#f4f5f2');
  if (persist) {
    try { localStorage.setItem('13forge-theme', theme); }
    catch { /* The selected theme still applies for this page view. */ }
  }
}

themeToggle.addEventListener('click', () => {
  applyTheme(document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark');
});

themePreference.addEventListener('change', (event) => {
  let hasSavedTheme = false;
  try { hasSavedTheme = Boolean(localStorage.getItem('13forge-theme')); }
  catch { /* Use the system preference when storage is unavailable. */ }
  if (!hasSavedTheme) applyTheme(event.matches ? 'dark' : 'light', false);
});

const revealItems = document.querySelectorAll('.intro, .replay-shell, .evidence-section, .architecture-section');
revealItems.forEach((item) => item.classList.add('reveal'));

const revealObserver = new IntersectionObserver((entries, observer) => {
  entries.forEach((entry) => {
    if (!entry.isIntersecting) return;
    entry.target.classList.add('is-visible');
    observer.unobserve(entry.target);
  });
}, { threshold: 0.08 });
revealItems.forEach((item) => revealObserver.observe(item));

const sections = document.querySelectorAll('#proof-replay, #evidence, #architecture');
const navLinks = Array.from(document.querySelectorAll('nav a'));
const navObserver = new IntersectionObserver((entries) => {
  entries.forEach((entry) => {
    if (!entry.isIntersecting) return;
    navLinks.forEach((link) => {
      const isCurrent = link.getAttribute('href') === `#${entry.target.id}`;
      if (isCurrent) link.setAttribute('aria-current', 'true');
      else link.removeAttribute('aria-current');
    });
  });
}, { rootMargin: '-35% 0px -55% 0px' });
sections.forEach((section) => navObserver.observe(section));

runButton.addEventListener('click', runReplay);
resetButton.addEventListener('click', resetReplay);
applyTheme(document.documentElement.dataset.theme, false);
resetReplay();
