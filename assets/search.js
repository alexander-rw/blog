(function () {
    var d = document.getElementById('search-dialog');
    var btn = document.getElementById('search-btn');
    var closeBtn = document.getElementById('search-close');
    var inp = document.getElementById('search-input');
    var res = document.getElementById('search-results');
    var idx = [];
    var idxRequest = null;

    // Fetch the index once, on first use, then re-run any query typed while it loaded.
    function loadIndex() {
        if (idxRequest) { return; }
        idxRequest = fetch('/search.json')
            .then(function (r) { return r.json(); })
            .then(function (data) { idx = data; renderResults(); })
            .catch(function () { idxRequest = null; });
    }

    function openDialog() {
        loadIndex();
        d.showModal();
        inp.value = '';
        res.innerHTML = '';
        inp.focus();
    }

    function escHtml(s) {
        return (s || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    }

    btn.addEventListener('click', openDialog);
    closeBtn.addEventListener('click', function () { d.close(); });
    d.addEventListener('click', function (e) { if (e.target === d) { d.close(); } });

    document.addEventListener('keydown', function (e) {
        if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
            e.preventDefault();
            openDialog();
        }
    });

    function renderResults() {
        var q = inp.value.toLowerCase().trim();
        if (!q || !idx.length) { res.innerHTML = ''; return; }
        var hits = idx.filter(function (p) {
            return (p.title + ' ' + (p.description || '') + ' ' + p.body)
                .toLowerCase()
                .includes(q);
        }).slice(0, 8);
        if (!hits.length) {
            res.innerHTML = '<li class="search-no-results">No results found</li>';
            return;
        }
        res.innerHTML = hits.map(function (p) {
            return '<li><a class="search-result" href="/blog/' + escHtml(p.slug) + '">'
                + '<span class="search-result-title">' + escHtml(p.title) + '</span>'
                + (p.date ? '<span class="search-result-date">' + escHtml(p.date) + '</span>' : '')
                + (p.description
                    ? '<p class="search-result-desc">' + escHtml(p.description) + '</p>'
                    : '')
                + '</a></li>';
        }).join('');
    }

    inp.addEventListener('input', renderResults);
}());
