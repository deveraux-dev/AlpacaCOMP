const runButton = document.querySelector('#run-check');
const resetButton = document.querySelector('#reset-check');
const decision = document.querySelector('#decision');
const gates = Array.from(document.querySelectorAll('.gate'));

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
  runButton.firstChild.textContent = 'Run safety checks ';
}

async function runReplay() {
  resetReplay();
  runButton.disabled = true;
  runButton.firstChild.textContent = 'Checking ';

  for (const gate of gates.slice(0, 3)) {
    gate.classList.add('is-checking');
    gate.querySelector('.gate-status').textContent = 'Checking';
    await delay(320);
    gate.classList.remove('is-checking');
    gate.classList.add('is-passed');
    gate.querySelector('.gate-status').textContent = 'Passed';
  }

  const riskGate = gates[3];
  riskGate.classList.add('is-checking');
  riskGate.querySelector('.gate-status').textContent = 'Calculating';
  await delay(520);
  riskGate.classList.remove('is-checking');
  riskGate.classList.add('is-refused');
  riskGate.querySelector('.gate-status').textContent = 'Refused';

  gates[4].classList.add('is-locked');
  gates[4].querySelector('.gate-status').textContent = 'Not reached';
  setDecision('is-refused', 'Order refused before Alpaca', '$2,525 maximum loss exceeds the $2,000 hard limit.');
  runButton.firstChild.textContent = 'Replay complete ';
  runButton.disabled = false;
}

runButton.addEventListener('click', runReplay);
resetButton.addEventListener('click', resetReplay);
resetReplay();
