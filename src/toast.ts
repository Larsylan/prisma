import './lib/theme.css';

import { mount } from 'svelte';

import { api, events } from './lib/api';
import { setLocale } from './lib/i18n/index.svelte';
import Toast from './Toast.svelte';

setLocale(await api.locale());
events.onLanguageChanged(setLocale);
mount(Toast, { target: document.getElementById('toast')! });
