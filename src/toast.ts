import './lib/theme.css';

import { mount } from 'svelte';

import Toast from './Toast.svelte';

mount(Toast, { target: document.getElementById('toast')! });
