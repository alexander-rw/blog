(function () {
  var btn = document.getElementById('theme-toggle');
  var html = document.documentElement;
  function applyTheme(theme) {
    html.setAttribute('data-theme', theme);
  }
  var stored = localStorage.getItem('theme');
  if (stored) {
    applyTheme(stored);
  } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
    applyTheme('dark');
  }
  btn.addEventListener('click', function () {
    var next = html.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
    localStorage.setItem('theme', next);
    applyTheme(next);
  });
}());
