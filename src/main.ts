import './lib/theme.css';

import { mount } from 'svelte';

import App from './App.svelte';
import { api, events } from './lib/api';
import { setLocale } from './lib/i18n/index.svelte';

// La langue est connue avant le premier affichage : l'interface n'apparaît jamais dans la mauvaise.
setLocale(await api.locale());
events.onLanguageChanged(setLocale);
mount(App, { target: document.getElementById('app')! });
