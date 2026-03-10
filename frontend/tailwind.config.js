/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./app.js"],
  safelist: [
    'btn', 'btn-primary', 'btn-secondary', 'btn-accent', 'btn-error', 'btn-success', 'btn-info', 'btn-warning', 'btn-outline', 'btn-ghost', 'btn-sm', 'btn-xs', 'btn-md', 'btn-lg', 'btn-circle', 'btn-square',
    'input', 'input-bordered', 'input-ghost', 'input-primary', 'input-secondary', 'input-accent', 'input-success', 'input-warning', 'input-info', 'input-error',
    'badge', 'badge-primary', 'badge-secondary', 'badge-accent', 'badge-ghost', 'badge-outline', 'badge-success', 'badge-error', 'badge-info', 'badge-warning',
    'select', 'select-bordered', 'table', 'table-zebra', 'textarea', 'textarea-bordered',
    'card', 'card-body', 'card-title', 'card-actions',
    'navbar', 'footer', 'modal', 'modal-box', 'modal-action', 'modal-backdrop',
    'form-control', 'label', 'label-text', 'label-text-alt',
    'alert', 'alert-info', 'alert-success', 'alert-warning', 'alert-error',
    'toast', 'toast-bottom', 'toast-center',
    { pattern: /^(badge|btn|input|alert)-/ },
  ],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: ["corporate", "light", "dark"],
    base: true,
    utils: true,
  },
}
