import { savePng } from '/src/core/export.js';

const root = document.documentElement;

document.querySelector('#theme').addEventListener('click', () => {
  const dark =
    root.dataset.theme === 'dark' || (!root.dataset.theme && matchMedia('(prefers-color-scheme: dark)').matches);
  root.dataset.theme = dark ? 'light' : 'dark';
});

document.querySelector('#mixed').indeterminate = true;

document.querySelector('#open-dialog').addEventListener('click', () => document.querySelector('#confirm').showModal());
document.querySelector('#open-command').addEventListener('click', () => document.querySelector('#commands').show());
document.querySelector('#open-drawer').addEventListener('click', () => document.querySelector('#drawer').showModal());
document.querySelector('#save-map').addEventListener('click', () => savePng(document.querySelector('#map'), 'map.png'));
