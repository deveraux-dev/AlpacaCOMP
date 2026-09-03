// Progressive enhancement: Show content immediately if JS fails, otherwise enable reveal animations
document.body.classList.add('js-enabled');

// Reveal animations on load and scroll
const reveals = document.querySelectorAll('.reveal');
function revealContent() {
  const windowHeight = window.innerHeight;
  const elementVisible = 150;
  reveals.forEach(reveal => {
    const elementTop = reveal.getBoundingClientRect().top;
    if (elementTop < windowHeight - elementVisible) {
      reveal.classList.add('visible');
    }
  });
}
window.addEventListener('scroll', revealContent);
// Trigger once on init
setTimeout(revealContent, 50);

// Shared proof metadata. The portal and print packet both read this file so
// teammate updates land in one place before deployment.
async function hydrateProofData() {
  try {
    const response = await fetch('./proof-data.json', { cache: 'no-store' });
    if (!response.ok) return;
    const proof = await response.json();
    document.querySelectorAll('[data-proof]').forEach((node) => {
      const key = node.getAttribute('data-proof');
      if (proof[key]) {
        node.textContent = proof[key];
      }
    });
  } catch (e) {}
}
hydrateProofData();


// Theme toggle logic
const themeBtn = document.getElementById('theme-toggle');
const iconMoon = document.getElementById('theme-icon-moon');
const iconSun = document.getElementById('theme-icon-sun');

function updateThemeIcon(theme) {
  if (theme === 'light') {
    iconMoon.style.display = 'block';
    iconSun.style.display = 'none';
  } else {
    iconMoon.style.display = 'none';
    iconSun.style.display = 'block';
  }
}

// Read current theme state
const initialTheme = document.documentElement.getAttribute('data-theme') || 'dark';
updateThemeIcon(initialTheme);

themeBtn.addEventListener('click', () => {
  const current = document.documentElement.getAttribute('data-theme') || 'dark';
  const next = current === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', next);
  try {
    localStorage.setItem('theme', next);
  } catch(e) {}
  updateThemeIcon(next);
});


// Replay logic
const runButton = document.querySelector('#run-check');
const resetButton = document.querySelector('#reset-check');
const decision = document.querySelector('#decision');

const delay = (milliseconds) => new Promise((resolve) => window.setTimeout(resolve, milliseconds));

function setDecision(state, title, detail) {
  decision.className = `decision ${state}`.trim();
  decision.querySelector('.decision-icon').textContent = state === 'is-refused' ? '×' : '?';
  decision.querySelector('strong').textContent = title;
  decision.querySelector('small').textContent = detail;
}

function resetReplay() {
  document.querySelectorAll('.gate').forEach((gate) => {
    const type = gate.dataset.gate;
    gate.className = type === 'broker' ? 'gate broker-gate is-locked' : 'gate';
    gate.querySelector('.gate-status').textContent = type === 'broker' ? 'Locked' : 'Waiting';
  });
  setDecision('', 'Awaiting replay', 'Run the checks to see the code-backed verdict.');
  runButton.disabled = false;
  runButton.firstChild.textContent = 'Run safety checks ';
}

async function runReplay() {
  resetReplay();
  runButton.disabled = true;
  runButton.firstChild.textContent = 'Checking ';

  // Select gates explicitly via attributes
  const sequence = [
    document.querySelector('.gate[data-gate="governor"]'),
    document.querySelector('.gate[data-gate="state"]'),
    document.querySelector('.gate[data-gate="oracle"]'),
    document.querySelector('.gate[data-gate="market"]'),
    document.querySelector('.gate[data-gate="geometry"]')
  ];

  for (const gate of sequence) {
    if (!gate) continue;
    gate.classList.add('is-checking');
    gate.querySelector('.gate-status').textContent = 'Checking';
    await delay(320);
    gate.classList.remove('is-checking');
    gate.classList.add('is-passed');
    gate.querySelector('.gate-status').textContent = 'Passed';
  }

  const riskGate = document.querySelector('.gate[data-gate="risk"]');
  if (riskGate) {
    riskGate.classList.add('is-checking');
    riskGate.querySelector('.gate-status').textContent = 'Calculating';
    await delay(520);
    riskGate.classList.remove('is-checking');
    riskGate.classList.add('is-refused');
    riskGate.querySelector('.gate-status').textContent = 'Refused';
  }

  const brokerGate = document.querySelector('.gate[data-gate="broker"]');
  if (brokerGate) {
    brokerGate.classList.add('is-locked');
    brokerGate.querySelector('.gate-status').textContent = 'Not reached';
  }

  setDecision('is-refused', 'Order refused before Alpaca', '$2,525 maximum loss exceeds the $2,000 hard limit.');
  runButton.firstChild.textContent = 'Replay complete ';
  runButton.disabled = false;
}

runButton.addEventListener('click', runReplay);
resetButton.addEventListener('click', resetReplay);
resetReplay();
