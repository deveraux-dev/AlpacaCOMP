const runButton = document.querySelector('#run-check');
const resetButton = document.querySelector('#reset-check');
const decision = document.querySelector('#decision');
const gates = Array.from(document.querySelectorAll('.gate'));

const delay = (milliseconds) => new Promise((resolve) => window.setTimeout(resolve, milliseconds));

function setDecision(state, title, detail) {
  decision.className = `decision ${state}`.trim();
  // Using lucide icon instead of text for 'X' or '?'
  if (state === 'is-refused') {
    decision.querySelector('.decision-icon').innerHTML = '<i data-lucide="x" width="20" height="20"></i>';
  } else {
    decision.querySelector('.decision-icon').innerHTML = '<i data-lucide="help-circle" width="20" height="20"></i>';
  }
  lucide.createIcons(); // Refresh icons
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
  runButton.firstChild.textContent = 'Run safety checks ';
}

async function runReplay() {
  resetReplay();
  runButton.disabled = true;
  runButton.firstChild.textContent = 'Checking ';

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
  runButton.firstChild.textContent = 'Replay complete ';
  runButton.disabled = false;
}

runButton.addEventListener('click', runReplay);
resetButton.addEventListener('click', resetReplay);
resetReplay();

// --- PREMIUM UI ENHANCEMENTS ---

// Initialize Icons
lucide.createIcons();

// Intersection Observer for Magical Scroll Reveal
const observerOptions = { threshold: 0.1, rootMargin: "0px 0px -50px 0px" };
const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      entry.target.classList.add('visible');
    }
  });
}, observerOptions);

document.querySelectorAll('.reveal').forEach(el => observer.observe(el));

// Web Audio API: High-end subtle click sound
function playPremiumClick() {
  try {
    const AudioContext = window.AudioContext || window.webkitAudioContext;
    if (!AudioContext) return;
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    
    osc.type = 'sine';
    osc.frequency.setValueAtTime(400, ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(100, ctx.currentTime + 0.05);
    
    gain.gain.setValueAtTime(0.05, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.05);
    
    osc.connect(gain);
    gain.connect(ctx.destination);
    
    osc.start();
    osc.stop(ctx.currentTime + 0.05);
  } catch (e) {}
}

// Theme Toggle Logic
const toggleBtn = document.getElementById('theme-toggle');
const body = document.body;
const themeIcon = document.getElementById('theme-icon');
const themeText = document.getElementById('theme-text');

const savedTheme = localStorage.getItem('theme');
if (savedTheme === 'light') {
  body.setAttribute('data-theme', 'light');
  themeIcon.setAttribute('data-lucide', 'sun');
  themeText.innerText = 'Light Mode';
}

toggleBtn.addEventListener('click', () => {
  playPremiumClick(); 
  
  if (body.getAttribute('data-theme') === 'light') {
    body.removeAttribute('data-theme');
    localStorage.setItem('theme', 'dark');
    themeIcon.setAttribute('data-lucide', 'moon');
    themeText.innerText = 'Dark Mode';
  } else {
    body.setAttribute('data-theme', 'light');
    localStorage.setItem('theme', 'light');
    themeIcon.setAttribute('data-lucide', 'sun');
    themeText.innerText = 'Light Mode';
  }
  lucide.createIcons();
});

// Mouse Tracking for Spotlight Glow Effect
document.body.addEventListener("pointermove", (e) => {
  const { currentTarget: el, clientX: x, clientY: y } = e;
  const { top: t, left: l, width: w, height: h } = el.getBoundingClientRect();
  el.style.setProperty('--mouse-x', (x - l));
  el.style.setProperty('--mouse-y', (y - t));
});
