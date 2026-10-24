// Shared helper for the generation wizard's uniform-schema pages.
// Every page's input AND output is the FULL answers.schema.json shape:
// a page renders only its own fields but must carry every other key
// through untouched, so the page-N-output-matches-page-N+1-input contract
// holds from project identity through options to review.
//
// Adapted from the reference wizard at 9ef4d03. DEFAULTS mirrors only the
// schema properties that declare a default. project_name has none and stays
// unset until prefill or the person supplies it. This file is not a
// wyvern-provided script; pages load it with a relative script tag.
(function (global) {
  "use strict";

  var DEFAULTS = {
    db: "sqlite",
    mcp: true,
    licence: "MIT",
  };

  function isPlainObject(v) {
    return !!v && typeof v === "object" && !Array.isArray(v);
  }

  // Every page's collectCurrentPageData() output is recorded as a NEW ENTRY ON
  // window.wyvern.stack -- it does NOT come back as window.wyvern.page_data.
  // Verified against a live session: after POST /api/wizard/navigate {action:
  // "next", data: {...}}, GET /api/wizard/state returns page_data === {} and
  // stack === [{page, data}]. Reading only page_data therefore threw away every
  // edit and left each page redisplaying config.prefill.
  function stackAnswers() {
    var wyvern = global.wyvern || {};
    var stack = Array.isArray(wyvern.stack) ? wyvern.stack : [];
    var merged = {};
    // Oldest entry first, so the most recently visited page wins. Each page emits
    // the full object, so the last entry normally decides everything; folding the
    // whole stack just backfills any key an intermediate page failed to carry.
    stack.forEach(function (entry) {
      if (!entry || !isPlainObject(entry.data)) return;
      Object.keys(entry.data).forEach(function (k) { merged[k] = entry.data[k]; });
    });
    return merged;
  }

  // Merge order (later wins): schema DEFAULTS < config.prefill < accumulated
  // stack data < page_data.
  //
  // - On page 1's first load the stack is empty, so prefill (the agent's
  //   best-guess answers, seeded into window.wyvern.config.prefill) wins
  //   over DEFAULTS.
  // - On every page after that (including a Back-revisit), the stack holds the
  //   FULL accumulated object returned by each previous page's
  //   collectCurrentPageData(), so it wins over both DEFAULTS and prefill --
  //   prefill only backfills keys the stack is missing, so navigation never
  //   loses an edit.
  // - page_data stays last in the order: it is empty in the wyvern build this
  //   was verified against, but if a future build does populate it, it is the
  //   more specific signal and should still win.
  function currentAnswers() {
    var wyvern = global.wyvern || {};
    var prefill = isPlainObject(wyvern.config && wyvern.config.prefill) ? wyvern.config.prefill : {};
    var stackData = stackAnswers();
    var pageData = isPlainObject(wyvern.page_data) ? wyvern.page_data : {};

    var merged = {};
    Object.keys(DEFAULTS).forEach(function (k) { merged[k] = DEFAULTS[k]; });
    Object.keys(prefill).forEach(function (k) { merged[k] = prefill[k]; });
    Object.keys(stackData).forEach(function (k) { merged[k] = stackData[k]; });
    Object.keys(pageData).forEach(function (k) { merged[k] = pageData[k]; });
    return merged;
  }

  // window.wyvern is populated ASYNCHRONOUSLY: wyvern-api.js's own bootstrap
  // fires a fetch on load and only assigns global.wyvern once that resolves.
  // Every page's inline script runs synchronously, well before that fetch
  // completes. ready() awaits wyvernWizardState() before resolving to
  // currentAnswers(), so callers never read window.wyvern before it exists.
  function ready() {
    var fetchState =
      typeof global.wyvernWizardState === "function"
        ? global.wyvernWizardState()
        : Promise.resolve(null);
    return fetchState.then(
      function () {
        return currentAnswers();
      },
      function () {
        return currentAnswers();
      }
    );
  }

  global.WizardAnswers = {
    DEFAULTS: DEFAULTS,
    stackAnswers: stackAnswers,
    currentAnswers: currentAnswers,
    ready: ready,
  };
})(window);
