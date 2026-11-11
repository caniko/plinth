(() => {
  const root = document.body;
  if (!root.classList.contains("company-site")) return;
  const reduced = matchMedia("(prefers-reduced-motion: reduce)");
  const narrow = matchMedia("(max-width: 760px)");
  const controls = [...document.querySelectorAll(".company-motion-toggle")];
  let paused = false;
  function updateMotion() {
    const state = root.dataset.motionEnabled !== "true" ? "off" : reduced.matches ? "reduced" : paused ? "paused" : document.hidden ? "hidden" : "playing";
    root.dataset.motionState = state;
    for (const control of controls) {
      control.hidden = state === "off" || state === "reduced";
      control.setAttribute("aria-pressed", String(paused));
      control.textContent = paused ? "Resume motion  ▷" : "Pause motion  Ⅱ";
    }
  }
  for (const control of controls) control.addEventListener("click", () => { paused = !paused; updateMotion(); });
  reduced.addEventListener("change", updateMotion);
  document.addEventListener("visibilitychange", updateMotion);
  updateMotion();

  const menu = document.querySelector(".company-menu");
  const navigation = document.querySelector(".company-nav-links");
  let open = false;
  function updateMenu() {
    if (!menu || !navigation) return;
    menu.hidden = !narrow.matches;
    navigation.hidden = narrow.matches && !open;
    menu.setAttribute("aria-expanded", String(narrow.matches && open));
    menu.textContent = open && narrow.matches ? "Close" : "Menu";
  }
  menu?.addEventListener("click", () => { open = !open; updateMenu(); });
  navigation?.addEventListener("click", event => { if (event.target.closest("a") && narrow.matches) { open = false; updateMenu(); } });
  document.addEventListener("keydown", event => {
    if (event.key === "Escape" && open) { open = false; updateMenu(); menu?.focus(); }
  });
  narrow.addEventListener("change", () => { open = false; updateMenu(); });
  updateMenu();
  root.dataset.companyReady = "true";
})();
