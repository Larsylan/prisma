/** Les champs de texte sont enregistrés après une courte pause de frappe, les autres réglages aussitôt. */
export const TEXT_SAVE_DELAY_MS = 400;

export type Save = (delay?: number) => void;

/** Regroupe des appels rapprochés en un seul, exécuté après `delay` millisecondes de calme. */
export function debounced(action: () => void): Save {
  let pending: ReturnType<typeof setTimeout> | undefined;
  return (delay = 0) => {
    clearTimeout(pending);
    pending = setTimeout(action, delay);
  };
}
