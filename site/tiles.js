// One entry per thing the library draws. `min` is the smallest tile it reads well in, in pixels,
// and `most` the largest in cells, past 1x1 or 2x1 for anything that would look sparse. `adds`
// holds the names `npx fluidither add` takes. `render` gets `pick`, which chooses among examples:
// the first on the opening board, a seeded one after that.

const GITHUB = 'https://github.com/vrnrn/fluidither';
const BROWSER_TESTS = 136;
const TESTS = BROWSER_TESTS + 36 + 81;

const FIELD_KINDS = ['noise', 'waves', 'plasma', 'ripples', 'tunnel', 'metaballs', 'orb', 'stars', 'aurora', 'contours', 'beams', 'eclipse', 'horizon', 'mesh', 'grid', 'ridges', 'silk', 'smoke'];
const MATRICES = ['bayer4', 'bayer2', 'bayer8', 'halftone', 'blue-noise'];

const heading = (text) => `<h2 class="tile-title">${text}</h2>`;

const stat = (label, value, change) => `
  <div class="fd-stat">
    <span class="fd-stat-label">${label}</span>
    <span class="fd-stat-value">${value}</span>
    <span class="fd-stat-change">${change}</span>
  </div>`;

const ranges = [
  { value: 'day', label: 'Day' },
  { value: 'week', label: 'Week' },
  { value: 'month', label: 'Month' },
];

const barExamples = [
  {
    names: 'Sales; Costs',
    day: ['2 4 3 6 5', '3 2 4 2 3'],
    week: ['3 7 2 9 5', '5 1 6 4 7'],
    month: ['6 5 8 7 9', '4 4 5 6 5'],
    labels: 'Mon Tue Wed Thu Fri',
    description: 'Sales and costs',
  },
  {
    names: 'Commits; Reviews',
    day: ['5 2 6 3 4', '2 3 2 4 1'],
    week: ['8 4 6 9 7', '3 6 2 5 4'],
    month: ['4 7 5 8 6', '6 3 7 2 5'],
    labels: 'Mon Tue Wed Thu Fri',
    description: 'Commits and reviews',
  },
  {
    names: 'Orders; Returns',
    day: ['7 3 5 8 4', '1 2 1 3 1'],
    week: ['9 6 8 5 7', '2 1 3 2 1'],
    month: ['5 8 6 9 7', '2 2 1 3 2'],
    labels: 'Mon Tue Wed Thu Fri',
    description: 'Orders and returns',
  },
];

function wireBars(tile) {
  const tabs = tile.querySelector('fd-tabs');
  const chart = tile.querySelector('fd-chart');
  const example = barExamples[Number(tile.dataset.example)];
  tabs.addEventListener('change', ({ detail }) => {
    chart.setAttribute('series', example[detail.value].join('; '));
  });
}

function wireChat(tile) {
  const log = tile.querySelector('.tile-chat');
  tile.querySelector('fd-composer').addEventListener('send', ({ detail }) => {
    const message = document.createElement('p');
    message.className = 'fd-message';
    message.dataset.from = 'me';
    message.textContent = detail.text;
    log.append(message);
    log.scrollTop = log.scrollHeight;
  });
}

function wireOpeners(tile) {
  for (const button of tile.querySelectorAll('[data-opens]')) {
    button.addEventListener('click', () => tile.querySelector(`#${button.dataset.opens}`).showModal());
  }
}

export const POOL = [
  {
    id: 'area',
    min: [330, 200],
    adds: ['chart', 'card', 'badge', 'feedback'],
    render(pick) {
      const example = pick([
        { label: 'Revenue', value: '$48,210', change: 'Up 12% this month', series: '2 6 3 8 5 9 7 11; 1 2 2 4 3 5 4 6', names: 'Visits; Signups', about: 'Visits and signups rising from January to August' },
        { label: 'Readers', value: '128,407', change: 'Up 8.3% this month', series: '4 5 7 6 9 8 11 13; 2 2 3 3 4 5 5 7', names: 'Readers; Subscribers', about: 'Readers and subscribers rising from January to August' },
        { label: 'Build minutes', value: '3,916', change: 'Down 6% this month', series: '11 9 10 7 8 6 6 5; 6 5 5 4 4 3 3 2', names: 'Queued; Running', about: 'Queued and running minutes falling from January to August' },
      ]);
      return `
        <div class="tile-row">
          ${stat(example.label, example.value, example.change)}
          <span class="fd-badge" data-tone="green">Live</span>
        </div>
        <fd-chart kind="area" series="${example.series}" names="${example.names}" labels="Jan Feb Mar Apr May Jun Jul Aug" aria-label="${example.about}"></fd-chart>`;
    },
  },
  {
    id: 'bars',
    min: [210, 210],
    adds: ['chart', 'tabs'],
    render(pick, tile) {
      const example = pick(barExamples);
      tile.dataset.example = barExamples.indexOf(example);
      return `
        <fd-tabs value="week">
          <fd-tab-list aria-label="Range">
            ${ranges.map(({ value, label }) => `<fd-tab value="${value}">${label}</fd-tab>`).join('')}
          </fd-tab-list>
        </fd-tabs>
        <fd-chart kind="bar" series="${example.week.join('; ')}" names="${example.names}" labels="${example.labels}" aria-label="${example.description} across the week"></fd-chart>`;
    },
    wire: wireBars,
  },
  {
    id: 'line',
    min: [210, 200],
    most: [2, 1],
    adds: ['chart'],
    render(pick) {
      const example = pick([
        { title: 'Response time', series: '4 6 3 8 5 7 4; 6 5 7 4 6 5 6', names: 'This week; Last week', about: 'Response time this week against last week' },
        { title: 'Open issues', series: '9 8 8 6 5 5 3; 7 7 8 8 6 7 6', names: 'Bugs; Requests', about: 'Bugs falling while requests hold steady' },
      ]);
      return `
        ${heading(example.title)}
        <fd-chart kind="line" series="${example.series}" names="${example.names}" labels="Mon Tue Wed Thu Fri Sat Sun" aria-label="${example.about}"></fd-chart>`;
    },
  },
  {
    id: 'donut',
    min: [210, 210],
    most: [1, 2],
    adds: ['chart'],
    render(pick) {
      const example = pick([
        { title: 'Traffic', series: '5 3 2', labels: 'Search Direct Social', about: 'Traffic by source' },
        { title: 'Storage', series: '7 4 1', labels: 'Images Video Other', about: 'Storage by file type' },
      ]);
      return `
        ${heading(example.title)}
        <fd-chart kind="donut" series="${example.series}" labels="${example.labels}" aria-label="${example.about}"></fd-chart>`;
    },
  },
  {
    id: 'radar',
    min: [210, 210],
    most: [1, 2],
    adds: ['chart'],
    render(pick) {
      const example = pick([
        { series: '5 8 3 7 6; 3 4 6 2 5', names: 'Plan; Actual', labels: 'Speed Cost Reach Trust Scale', about: 'Plan against actual' },
        { series: '7 5 6 4 8; 4 6 3 7 5', names: 'Ours; Theirs', labels: 'Docs Tests Size Speed Reach', about: 'Our scores against theirs' },
      ]);
      return `<fd-chart kind="radar" series="${example.series}" names="${example.names}" labels="${example.labels}" aria-label="${example.about}"></fd-chart>`;
    },
  },
  {
    id: 'heatmap',
    min: [210, 200],
    most: [2, 1],
    adds: ['chart'],
    render(pick) {
      const example = pick([
        { title: 'Busiest hours', series: '1 4 2 6 3 5; 7 3 9 2 6 4; 2 8 5 4 7 1; 4 2 6 8 3 5' },
        { title: 'Deploys by day', series: '3 1 5 2 6 4; 2 7 3 8 1 5; 6 4 2 5 9 3; 1 5 7 3 4 2' },
      ]);
      return `
        ${heading(example.title)}
        <fd-chart kind="heatmap" series="${example.series}" aria-label="${example.title}, darker cells are higher"></fd-chart>`;
    },
  },
  {
    id: 'gauge',
    min: [210, 200],
    most: [1, 1],
    adds: ['chart'],
    render(pick) {
      const example = pick([
        { title: 'Quota', used: 62, name: 'Used' },
        { title: 'Disk', used: 81, name: 'Full' },
        { title: 'Battery', used: 37, name: 'Left' },
      ]);
      return `
        <div class="tile-row">${heading(example.title)}<span class="tile-note">${example.used}% ${example.name.toLowerCase()}</span></div>
        <fd-chart kind="gauge" series="${example.used} 100" names="${example.name}" aria-label="${example.used} percent ${example.name.toLowerCase()}"></fd-chart>`;
    },
  },
  {
    id: 'form',
    min: [210, 210],
    most: [1, 1],
    adds: ['input', 'switch', 'check', 'slider', 'button'],
    render(pick) {
      const example = pick([
        { name: 'Ada Lovelace', toggle: 'Sync', check: 'Send a weekly summary', level: 62, about: 'Volume' },
        { name: 'Grace Hopper', toggle: 'Autosave', check: 'Keep older versions', level: 38, about: 'Quality' },
        { name: 'Katherine Johnson', toggle: 'Alerts', check: 'Only when I am mentioned', level: 74, about: 'Brightness' },
      ]);
      return `
        <label class="fd-field">Name <input class="fd-input" value="${example.name}" /></label>
        <label class="fd-choice"><input class="fd-switch" type="checkbox" role="switch" checked /> ${example.toggle}</label>
        <label class="fd-choice"><input class="fd-check" type="checkbox" checked /> ${example.check}</label>
        <fd-slider><input type="range" min="0" max="100" value="${example.level}" aria-label="${example.about}" /></fd-slider>
        <div class="tile-row tile-end">
          <button class="fd-button" data-look="soft" type="button">Cancel</button>
          <button class="fd-button" type="button">Save</button>
        </div>`;
    },
  },
  {
    id: 'calendar',
    min: [210, 210],
    most: [1, 1],
    compact: 280,
    adds: ['calendar'],
    render(pick) {
      const value = pick(['2026-10-14', '2026-11-03', '2026-12-21', '2027-02-09']);
      return `<fd-calendar value="${value}" aria-label="Pick a day"></fd-calendar>`;
    },
  },
  {
    id: 'field',
    min: [200, 200],
    weight: 4,
    bare: true,
    adds: ['field'],
    render(pick) {
      const kind = pick(['aurora', ...FIELD_KINDS]);
      const matrix = pick(MATRICES);
      return `<fd-field kind="${kind}" matrix="${matrix}"></fd-field>`;
    },
  },
  {
    id: 'gradient',
    min: [200, 200],
    weight: 3,
    bare: true,
    adds: ['gradient'],
    render(pick) {
      const example = pick([
        { mode: 'linear', angle: 90, from: 0, to: 0.7, warp: 0.3 },
        { mode: 'radial', angle: 0, from: 0.75, to: 0, warp: 0.1 },
        { mode: 'conic', angle: 0, from: 0, to: 0.7, warp: 0.1 },
        { mode: 'linear', angle: 135, from: 0.05, to: 0.65, warp: 0.5 },
      ]);
      const matrix = pick(MATRICES);
      return `<fd-dither-gradient mode="${example.mode}" angle="${example.angle}" from="${example.from}" to="${example.to}" warp="${example.warp}" matrix="${matrix}" speed="0.5"></fd-dither-gradient>`;
    },
  },
  {
    id: 'globe',
    min: [210, 210],
    weight: 3,
    bare: true,
    adds: ['world'],
    render: () => '<fd-world kind="globe" aria-label="A dithered globe, turning"></fd-world>',
  },
  {
    id: 'map',
    min: [400, 200],
    weight: 2,
    bare: true,
    adds: ['world'],
    render: () => '<fd-world kind="map" aria-label="A dotted sketch of the world"></fd-world>',
  },
  {
    id: 'image',
    min: [200, 200],
    weight: 2,
    bare: true,
    adds: ['image'],
    render(pick) {
      const matrix = pick(['halftone', 'bayer8', 'blue-noise']);
      return `<fd-dither-image src="sphere.svg" alt="A lit sphere redrawn in dots" matrix="${matrix}"></fd-dither-image>`;
    },
  },
  {
    id: 'chat',
    min: [300, 210],
    adds: ['chat', 'composer', 'text'],
    render(pick) {
      const example = pick([
        ['Can you draft the report?', 'Sure, starting now.', 'Make it shorter'],
        ['Is the invoice for Atlas Print out?', 'Sent this morning. I copied Dana.', 'Thanks, send a reminder Friday'],
        ['Which tests are failing?', 'Two, both in the date picker.', 'Show me the first one'],
      ]);
      return `
        <div class="tile-chat">
          <p class="fd-message">${example[0]}</p>
          <p class="fd-message" data-from="me">${example[1]}</p>
        </div>
        <span class="fd-thinking"><span class="fd-loader" aria-hidden="true"></span> <fd-shimmer words="Thinking; Reading; Writing">Thinking</fd-shimmer></span>
        <fd-composer suggestion="${example[2]}">
          <form class="fd-composer">
            <textarea class="fd-input" rows="1" aria-label="Message"></textarea>
            <button class="fd-button" type="submit">Send</button>
          </form>
        </fd-composer>`;
    },
    wire: wireChat,
  },
  {
    id: 'accordion',
    min: [210, 210],
    most: [2, 1],
    adds: ['accordion'],
    render(pick) {
      const group = `faq-${pick([1, 2, 3])}`;
      const rows = pick([
        [
          ['What is dithering?', 'Mixing two colors in a fine pattern so they read as a tone.'],
          ['Why a spring?', 'It keeps its speed when the target moves, so motion never jerks.'],
          ['Does it need a framework?', 'No. These are real elements with a little CSS.'],
        ],
        [
          ['Will it pass a strict CSP?', 'Yes. There are no inline styles and no data images.'],
          ['What about reduced motion?', 'Every moving part respects it and draws a still frame.'],
          ['Which browsers are tested?', 'Chromium. Firefox and Safari have not been run yet.'],
        ],
      ]);
      return rows
        .map(([question, answer], index) => `
          <details class="fd-accordion" name="${group}"${index === 0 ? ' open' : ''}>
            <summary>${question}</summary>
            <div class="fd-accordion-body">${answer}</div>
          </details>`)
        .join('');
    },
  },
  {
    id: 'table',
    min: [400, 200],
    most: [2, 1],
    adds: ['table'],
    render(pick) {
      const example = pick([
        { title: 'Recent invoices', head: ['Invoice', 'Client', 'Amount'], rows: [['INV-1021', 'Harbour Coffee', '$1,204.00'], ['INV-1022', 'Atlas Print', '$98.50'], ['INV-1023', 'North Mill', '$3,410.75']] },
        { title: 'Open orders', head: ['Order', 'Carrier', 'Weight'], rows: [['8841', 'Fieldwork Freight', '12.4 kg'], ['8846', 'Lantern Post', '0.7 kg'], ['8852', 'Harbour Lines', '38.1 kg']] },
      ]);
      const [first, second, third] = example.head;
      return `
        <table class="fd-table">
          <caption>${example.title}</caption>
          <thead><tr><th scope="col">${first}</th><th scope="col">${second}</th><th scope="col" data-number>${third}</th></tr></thead>
          <tbody>${example.rows.map(([a, b, c]) => `<tr><td>${a}</td><td>${b}</td><td data-number>${c}</td></tr>`).join('')}</tbody>
        </table>`;
    },
  },
  {
    id: 'sparkline',
    min: [200, 170],
    most: [1, 1],
    adds: ['sparkline', 'feedback'],
    render(pick) {
      const example = pick([
        { label: 'Signups', value: '1,873', change: 'Up 9% this week', values: '4 6 5 8 7 11 9 13 12 16', about: 'Signups rose over ten days' },
        { label: 'Refunds', value: '41', change: 'Down 3% this week', values: '9 8 9 7 8 6 7 5 6 4', about: 'Refunds fell over ten days' },
      ]);
      return `
        ${stat(example.label, example.value, example.change)}
        <fd-sparkline values="${example.values}" aria-label="${example.about}"></fd-sparkline>`;
    },
  },
  {
    id: 'uptime',
    min: [200, 130],
    most: [2, 1],
    adds: ['misc'],
    render(pick) {
      const example = pick([
        { title: 'API uptime', note: '99.94% over 30 days', partial: [7], down: [] },
        { title: 'Mail delivery', note: '99.71% over 30 days', partial: [4, 19], down: [12] },
      ]);
      const days = Array.from({ length: 30 }, (_, day) => {
        if (example.down.includes(day)) return '<i data-state="down"></i>';
        return example.partial.includes(day) ? '<i data-state="partial"></i>' : '<i></i>';
      }).join('');
      return `
        ${heading(example.title)}
        <div class="fd-uptime" role="img" aria-label="${example.note}">${days}</div>
        <p class="tile-note">${example.note}</p>`;
    },
  },
  {
    id: 'steps',
    min: [200, 170],
    most: [1, 1],
    adds: ['steps'],
    render(pick) {
      const example = pick([
        ['Read the brief', 'Gather sources', 'Write the draft', 'Review'],
        ['Pack the order', 'Hand to carrier', 'Out for delivery', 'Delivered'],
        ['Copy the parts', 'Paste the tags', 'Serve the folder', 'Ship it'],
      ]);
      return `
        <ol class="fd-steps">
          <li data-state="done">${example[0]}</li>
          <li data-state="done">${example[1]}</li>
          <li data-state="active">${example[2]}</li>
          <li>${example[3]}</li>
        </ol>`;
    },
  },
  {
    id: 'question',
    min: [210, 190],
    most: [1, 1],
    adds: ['question'],
    render(pick) {
      const example = pick([
        { ask: 'Which format should this take?', one: 'A short report', two: 'Slides' },
        { ask: 'When should this run?', one: 'Every morning', two: 'Only on Mondays' },
      ]);
      const name = `choice-${pick([1, 2, 3])}`;
      return `
        <fieldset class="fd-question">
          <legend>${example.ask}</legend>
          <label class="fd-choice"><input class="fd-radio" type="radio" name="${name}" checked /> ${example.one}</label>
          <label class="fd-choice"><input class="fd-radio" type="radio" name="${name}" /> ${example.two}</label>
          <button class="fd-button" data-density="compact" type="button">Answer</button>
        </fieldset>`;
    },
  },
  {
    id: 'buttons',
    min: [210, 170],
    most: [1, 1],
    adds: ['button', 'tooltip'],
    render: () => `
      <div class="tile-wrap">
        <button class="fd-button" type="button">Solid</button>
        <button class="fd-button" data-look="soft" type="button">Soft</button>
        <button class="fd-button" data-look="outline" type="button">Outline</button>
        <button class="fd-button" data-look="ghost" type="button">Ghost</button>
        <button class="fd-button" type="button" aria-busy="true" disabled>Saving</button>
        <fd-tooltip text="Every hover runs on a spring"><button class="fd-button" data-look="soft" type="button">Hover me</button></fd-tooltip>
      </div>`,
  },
  {
    id: 'status',
    min: [210, 170],
    most: [2, 1],
    adds: ['badge', 'banner', 'button'],
    render(pick) {
      const example = pick([
        { line: 'Your trial ends in three days.', action: 'Upgrade' },
        { line: 'Two invoices need a review.', action: 'Open' },
      ]);
      return `
        <div class="tile-wrap">
          <span class="fd-badge">Draft</span>
          <span class="fd-badge" data-tone="green">Live</span>
          <span class="fd-badge" data-tone="orange">Review</span>
        </div>
        <div class="fd-banner" data-tone="orange" role="status">
          <span>${example.line}</span>
          <button class="fd-button" data-look="soft" data-density="compact" type="button">${example.action}</button>
        </div>`;
    },
  },
  {
    id: 'files',
    min: [200, 150],
    most: [1, 1],
    adds: ['file', 'feedback'],
    render(pick) {
      const example = pick([
        { type: 'pdf', name: 'Report.pdf', size: '2.4 MB', share: 62 },
        { type: 'csv', name: 'Orders.csv', size: '814 KB', share: 38 },
        { type: 'zip', name: 'Assets.zip', size: '41.7 MB', share: 87 },
      ]);
      return `
        <span class="fd-file"><span class="fd-file-type">${example.type}</span><span>${example.name} <span class="fd-file-size">${example.size}</span></span></span>
        <progress class="fd-progress" value="${example.share}" max="100" aria-label="Uploading ${example.name}"></progress>
        <p class="tile-note">${example.share}% uploaded</p>`;
    },
  },
  {
    id: 'pickers',
    min: [250, 170],
    most: [1, 1],
    adds: ['menu', 'select', 'combobox', 'button'],
    render: () => `
      <div class="fd-field">
        <fd-select name="fruit" value="pear" aria-label="Fruit" placeholder="Choose a fruit">
          <fd-list>
            <fd-item value="apple">Apple</fd-item>
            <fd-item value="cherry" disabled>Cherry</fd-item>
            <fd-item value="pear">Pear</fd-item>
            <fd-item value="plum">Plum</fd-item>
          </fd-list>
        </fd-select>
      </div>
      <div class="fd-field">
        <fd-combobox name="city" aria-label="City" placeholder="Search cities">
          <fd-list>
            <fd-item value="lis">Lisbon</fd-item>
            <fd-item value="osl">Oslo</fd-item>
            <fd-item value="tok">Tokyo</fd-item>
          </fd-list>
        </fd-combobox>
      </div>
      <fd-menu>
        <button class="fd-button" data-look="outline" type="button">Actions</button>
        <fd-list>
          <fd-item value="rename">Rename</fd-item>
          <fd-item value="duplicate">Duplicate</fd-item>
          <fd-item value="archive" disabled>Archive</fd-item>
        </fd-list>
      </fd-menu>`,
  },
  {
    id: 'dialog',
    min: [200, 150],
    most: [1, 1],
    adds: ['dialog', 'drawer', 'button'],
    render: () => `
      ${heading('Dialog and drawer')}
      <p class="tile-note">Real dialogs, on a dithered screen.</p>
      <div class="tile-wrap tile-end">
        <button class="fd-button" data-look="soft" type="button" data-opens="tile-drawer">Drawer</button>
        <button class="fd-button" type="button" data-opens="tile-dialog">Dialog</button>
      </div>
      <dialog class="fd-dialog" id="tile-dialog" aria-labelledby="tile-dialog-title">
        <h3 class="fd-dialog-title" id="tile-dialog-title">Delete this report?</h3>
        <p class="fd-dialog-text">This can't be undone.</p>
        <form class="fd-dialog-actions" method="dialog">
          <button class="fd-button" data-look="soft" value="cancel">Cancel</button>
          <button class="fd-button" value="delete">Delete</button>
        </form>
      </dialog>
      <dialog class="fd-dialog fd-drawer" id="tile-drawer" aria-labelledby="tile-drawer-title">
        <h3 class="fd-dialog-title" id="tile-drawer-title">Filters</h3>
        <p class="fd-dialog-text">A panel that slides in from the edge.</p>
        <form class="fd-dialog-actions" method="dialog"><button class="fd-button" data-look="soft">Close</button></form>
      </dialog>`,
    wire: wireOpeners,
  },
  {
    id: 'command',
    min: [200, 150],
    most: [1, 1],
    adds: ['command', 'button'],
    render: () => `
      ${heading('Command menu')}
      <p class="tile-note">This one can drive the page.</p>
      <div class="tile-wrap tile-end">
        <button class="fd-button" type="button" data-command>Open</button>
      </div>
      <fd-command aria-label="Commands" placeholder="Type a command">
        <fd-list>
          <fd-item value="randomize">Randomize the board</fd-item>
          <fd-item value="theme">Switch theme</fd-item>
          <fd-item value="github">Open GitHub</fd-item>
        </fd-list>
      </fd-command>`,
    wire(tile, page) {
      const menu = tile.querySelector('fd-command');
      tile.querySelector('[data-command]').addEventListener('click', () => menu.show());
      menu.addEventListener('pick', ({ detail }) => {
        if (detail.value === 'randomize') page.randomize();
        else if (detail.value === 'theme') page.toggleTheme();
        else location.assign(GITHUB);
      });
    },
  },
  {
    id: 'text',
    min: [210, 150],
    most: [2, 1],
    replay: true,
    adds: ['text', 'button'],
    render(pick) {
      const word = pick(['Scrambled into place', 'Printed on arrival', 'Set in loose type']);
      return `<p class="tile-big"><fd-scramble>${word}</fd-scramble></p>`;
    },
  },
  {
    id: 'compare',
    min: [210, 200],
    bare: true,
    adds: ['compare', 'field'],
    render(pick) {
      const [kind, left, right] = pick([
        ['aurora', 'bayer2', 'blue-noise'],
        ['eclipse', 'bayer8', 'halftone'],
        ['orb', 'halftone', 'blue-noise'],
      ]);
      return `
        <fd-compare aria-label="Compare ${left} against ${right}">
          <fd-field kind="${kind}" speed="0" matrix="${left}"></fd-field>
          <fd-field kind="${kind}" speed="0" matrix="${right}"></fd-field>
        </fd-compare>`;
    },
  },
  {
    id: 'marquee',
    min: [400, 130],
    most: [2, 1],
    adds: ['marquee'],
    render(pick) {
      const words = pick([
        ['No framework', 'No runtime dependencies', 'No build step', 'No inline styles', 'No data images'],
        ['Bayer', 'Halftone', 'Blue noise', 'Plasma', 'Aurora', 'Metaballs'],
      ]);
      return `<fd-marquee class="tile-big" aria-label="${words.join(', ')}">${words.map((word) => `<span>${word}</span>`).join('')}</fd-marquee>`;
    },
  },
  {
    id: 'sidebar',
    min: [250, 170],
    most: [1, 1],
    adds: ['sidebar', 'nav'],
    render(pick) {
      const [first, second, third] = pick([
        ['Inbox', 'Sent', 'Archive'],
        ['Overview', 'Activity', 'Billing'],
      ]);
      return `
        <fd-sidebar label="menu">
          <fd-nav aria-label="Menu" in-page><a href="#one" aria-current="page">${first}</a><a href="#two">${second}</a><a href="#three">${third}</a></fd-nav>
        </fd-sidebar>`;
    },
  },
  {
    id: 'loading',
    min: [200, 150],
    most: [1, 1],
    adds: ['feedback'],
    render: () => `
      <p class="tile-note"><span class="fd-loader" role="status" aria-label="Loading"></span> Loading reports</p>
      <hr class="fd-divider" />
      <span class="fd-skeleton" aria-hidden="true"></span>
      <span class="fd-skeleton tile-short" aria-hidden="true"></span>`,
  },
  {
    id: 'missing',
    min: [200, 190],
    most: [1, 1],
    adds: ['not-found', 'text', 'button'],
    render: () => `
      <div class="fd-not-found">
        <p class="fd-not-found-code"><fd-dissolve hold="0.1 0.7">404</fd-dissolve></p>
        <p>This page has gone missing.</p>
      </div>`,
  },
  {
    id: 'empty',
    min: [200, 150],
    most: [1, 1],
    adds: ['misc'],
    render(pick) {
      const [title, line] = pick([
        ['No reports yet', 'New reports will appear here.'],
        ['Nothing to review', 'You have read everything.'],
      ]);
      return `<div class="fd-empty"><h2 class="fd-empty-title">${title}</h2><p>${line}</p></div>`;
    },
  },
  {
    id: 'glide',
    min: [200, 200],
    most: [1, 2],
    weight: 3,
    adds: ['hover'],
    render(pick) {
      const [title, rows] = pick([
        ['Move across these', [['Inbox', '12'], ['Drafts', '3'], ['Sent', ''], ['Archive', ''], ['Trash', '']]],
        ['One highlight', [['Overview', ''], ['Activity', '7'], ['Billing', ''], ['Members', '24'], ['Settings', '']]],
      ]);
      return `
        ${heading(title)}
        <fd-hover class="tile-glide" nearest axis="y">
          ${rows.map(([name, count]) => `<div>${name}<span>${count}</span></div>`).join('')}
        </fd-hover>`;
    },
  },
  {
    id: 'checks',
    min: [200, 190],
    most: [1, 1],
    adds: ['check-group'],
    render(pick) {
      const [legend, rows] = pick([
        ['Fruit', ['Apples', 'Bananas', 'Cherries', 'Dates']],
        ['Notify me', ['Mentions', 'Replies', 'Reviews', 'Releases']],
      ]);
      const group = `checks-${pick([1, 2, 3])}`;
      return `
        <fieldset class="fd-check-group">
          <legend class="tile-title">${legend}</legend>
          ${rows.map((row, index) => `<label class="fd-choice"><input class="fd-check" type="checkbox" name="${group}"${index < 2 ? ' checked' : ''} /> ${row}</label>`).join('')}
        </fieldset>`;
    },
  },
  {
    id: 'thinking',
    min: [210, 200],
    most: [1, 2],
    adds: ['thinking-steps', 'steps', 'badge'],
    render(pick) {
      const [title, done, active, source] = pick([
        ['Research', 'Read the brief', 'Write the draft', 'docs'],
        ['Deploying', 'Ran the tests', 'Publishing the site', 'ci'],
      ]);
      return `
        <details class="fd-thinking-steps" open>
          <summary>${title}</summary>
          <ol class="fd-steps">
            <li data-state="done">${done}<span class="fd-sources"><span class="fd-badge">${source}</span></span></li>
            <li data-state="active">${active}</li>
            <li>Review</li>
          </ol>
        </details>`;
    },
  },
  {
    id: 'questions',
    min: [240, 232],
    most: [1, 2],
    replay: true,
    adds: ['questions'],
    render(pick) {
      const stem = `ask-${pick([1, 2, 3])}`;
      return `
        <fd-questions done="Thanks. Press replay to go again." data-density="compact">
          <fieldset class="fd-question">
            <legend>What are you making?</legend>
            <label class="fd-choice"><input class="fd-radio" type="radio" name="${stem}-a" value="site" /> A website</label>
            <label class="fd-choice"><input class="fd-radio" type="radio" name="${stem}-a" value="app" /> A desktop app</label>
          </fieldset>
          <fieldset class="fd-question">
            <legend>Which target?</legend>
            <label class="fd-choice"><input class="fd-radio" type="radio" name="${stem}-b" value="web" /> The web</label>
            <label class="fd-choice"><input class="fd-radio" type="radio" name="${stem}-b" value="gpui" /> GPUI</label>
          </fieldset>
        </fd-questions>`;
    },
  },
  {
    id: 'contact',
    min: [230, 150],
    most: [2, 1],
    adds: ['input-group'],
    render: () => `
      ${heading('Your details')}
      <div class="fd-input-group">
        <label class="fd-input-row"><span>Name</span><input class="fd-input" value="Ada Lovelace" /></label>
        <label class="fd-input-row"><span>Email</span><input class="fd-input" type="email" value="ada@" aria-invalid="true" /></label>
      </div>
      <p class="tile-note">That doesn't look like an address.</p>`,
  },
  {
    id: 'picker',
    min: [210, 232],
    most: [1, 2],
    adds: ['color-picker'],
    render: () => '<fd-color-picker value="#fc9e47" aria-label="Accent"></fd-color-picker>',
  },
  {
    id: 'color',
    min: [210, 150],
    most: [1, 1],
    adds: ['date-picker', 'swatches', 'color'],
    render: () => `
      <fd-date-picker name="due" value="2026-10-30" aria-label="Due date"></fd-date-picker>
      <div class="tile-wrap">
        <fd-swatches colors="#FC9E47 #B25900 #6DD17F #A8A498 #5F5C54" value="#FC9E47" aria-label="Accent color"></fd-swatches>
        <input class="fd-color" type="color" value="#fc9e47" aria-label="Custom color" />
      </div>`,
  },
  {
    id: 'pieces',
    min: [200, 130],
    most: [1, 1],
    adds: ['misc', 'bloom', 'badge'],
    render(pick) {
      const [initials, name] = pick([
        ['AL', 'Ada Lovelace'],
        ['GH', 'Grace Hopper'],
        ['KJ', 'Katherine Johnson'],
      ]);
      return `
        <div class="tile-wrap tile-middle">
          <span class="fd-avatar" role="img" aria-label="${name}">${initials}</span>
          <span class="fd-bloom" data-tone="orange"><span class="fd-badge" data-tone="orange">Glowing</span></span>
          <nav class="fd-dots" aria-label="Slides">
            <button type="button" aria-label="Slide 1"></button>
            <button type="button" aria-label="Slide 2" aria-current="true"></button>
            <button type="button" aria-label="Slide 3"></button>
          </nav>
        </div>`;
    },
  },
  {
    id: 'code',
    min: [400, 200],
    adds: ['tabs', 'scroll'],
    render: () => `
      <fd-tabs value="web" look="underline">
        <fd-tab-list aria-label="Target">
          <fd-tab value="web">Web</fd-tab>
          <fd-tab value="gpui">GPUI</fd-tab>
        </fd-tab-list>
        <fd-tab-panel value="web">
          <pre class="fd-scroll tile-code" tabindex="0" aria-label="The same screen for the web"><code>&lt;fd-tabs value="week"&gt;
  &lt;fd-tab-list aria-label="Range"&gt;
    &lt;fd-tab value="day"&gt;Day&lt;/fd-tab&gt;
    &lt;fd-tab value="week"&gt;Week&lt;/fd-tab&gt;
  &lt;/fd-tab-list&gt;
&lt;/fd-tabs&gt;

&lt;fd-dither-gradient speed="0.4"&gt;&lt;/fd-dither-gradient&gt;</code></pre>
        </fd-tab-panel>
        <fd-tab-panel value="gpui" hidden>
          <pre class="fd-scroll tile-code" tabindex="0" aria-label="The same screen for GPUI"><code>div()
    .child(Tabs::new("range", ["Day", "Week"]))
    .child(
        DitherGradient::new("band", Gradient::default())
            .speed(0.4)
            .h(px(120.))
            .w_full(),
    )</code></pre>
        </fd-tab-panel>
      </fd-tabs>`,
  },
  {
    id: 'proof',
    min: [200, 150],
    most: [1, 1],
    weight: 2,
    replay: true,
    adds: ['text', 'feedback', 'button'],
    render(pick) {
      const { value, digits, unit, line } = pick([
        { value: 4.2, digits: 1, unit: ' KB', line: 'Tokens and an animated gradient, gzipped' },
        { value: TESTS, digits: 0, unit: ' tests', line: 'Including cell by cell parity between targets' },
        { value: 0, digits: 0, unit: ' dependencies', line: 'At runtime, on the web' },
        { value: BROWSER_TESTS, digits: 0, unit: ' browser tests', line: 'Under the strict CSP, with axe in both themes' },
      ]);
      return `
        <p class="tile-figure"><fd-count value="${value}" digits="${digits}"></fd-count>${unit}</p>
        <p class="tile-note">${line}</p>`;
    },
  },
];

// The first board, the same for everyone: the reference layout on the right, the hero beside it.
const OPENING = [
  { id: 'area', x: 2, y: 0, width: 2, height: 1 },
  { id: 'field', x: 4, y: 0, width: 1, height: 1 },
  { id: 'form', x: 2, y: 1, width: 1, height: 1 },
  { id: 'calendar', x: 3, y: 1, width: 1, height: 1 },
  { id: 'bars', x: 4, y: 1, width: 1, height: 1 },
  { id: 'glide', x: 0, y: 2, width: 1, height: 1 },
  { id: 'proof', x: 1, y: 2, width: 1, height: 1 },
  { id: 'globe', x: 2, y: 2, width: 1, height: 1 },
  { id: 'chat', x: 3, y: 2, width: 2, height: 1 },
];

/** The first board for `columns`. On the full wall it is the set above, otherwise its tiles in reading order. */
export function opening(columns, wall) {
  if (columns === wall.columns) {
    return { hero: { x: 0, y: 0, width: 2, height: 2 }, tiles: OPENING, rows: wall.rows };
  }
  const hero = { x: 0, y: 0, width: Math.min(columns, 2), height: 1 };
  let x = hero.width;
  let y = 0;
  const tiles = ['area', 'bars', 'field', 'calendar', 'form', 'chat', 'globe', 'glide'].map((id) => {
    const width = OPENING.find((tile) => tile.id === id).width === 2 ? Math.min(columns, 2) : 1;
    if (x + width > columns) {
      x = 0;
      y += 1;
    }
    const tile = { id, x, y, width, height: 1 };
    x += width;
    return tile;
  });
  return { hero, tiles, rows: y + 1 };
}
