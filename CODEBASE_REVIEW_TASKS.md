# Codebase-Review: konkrete Aufgabenvorschläge

## 1) Aufgabe: Tippfehler korrigieren
**Fundstelle:** `README.md`, Keybindings-Tabelle: „Open File browser (rofi)“.  
**Problem:** „File browser“ ist hier sehr wahrscheinlich ein Tippfehler/Bezeichnungsfehler, da `rofi` ein App-Launcher ist und kein Dateimanager.

**Vorschlag für Task:**
- In der Keybindings-Tabelle den Eintrag `Mod + D` von „Open File browser (rofi)“ auf „Open app launcher (rofi)“ ändern.
- Optional in DE-Abschnitt analog „Anwendungsstarter (rofi)“ formulieren.

---

## 2) Aufgabe: Programmierfehler beheben
**Fundstelle:** `src/wm.rs`, Funktion `close_window()` und `is_fullscreen()`.

**Problem:** Mehrere `.unwrap()` auf X11-Requests können den Window-Manager bei Protokoll-/I/O-Fehlern abstürzen lassen.
Ein WM sollte hier fehlertolerant sein und nicht wegen einzelner Client-/X11-Fehler terminieren.

**Vorschlag für Task:**
- `close_window()` und `is_fullscreen()` auf fehlertolerantes Fehlerhandling umstellen (`if let Ok(...)`, `match`, Logging statt Panic).
- Bei Fehlern: best-effort Verhalten (z. B. bei `close_window` Fallback auf `kill_client`, falls `WM_DELETE_WINDOW` nicht verfügbar ist).
- Kurze Regression-Tests für Hilfslogik extrahieren (z. B. Atom-Auswertung in pure Funktion), damit Panic-Fälle reproduzierbar getestet werden können.

---

## 3) Aufgabe: Kommentar-/Doku-Unstimmigkeit korrigieren
**Fundstelle:** `README.md` vs. `src/wm.rs`.

**Problem:** Dokumentation nennt bei `Mod + B` „Open browser (firefox)“, der Code startet aber `firefox-esr`.
Damit können Nutzer auf Nicht-Debian-Systemen verwirrt sein oder inkonsistente Erwartungen haben.

**Vorschlag für Task:**
- README und Code auf einen gemeinsamen Browser-Startmechanismus vereinheitlichen.
- Entweder Doku explizit auf `firefox-esr` anpassen **oder** Code auf robusten Fallback umstellen (`firefox-esr` → `firefox`), und Doku entsprechend beschreiben.

---

## 4) Aufgabe: Test verbessern
**Fundstelle:** `src/state.rs` Tests.

**Problem:** Die State-Tests decken Fokusnavigation gut ab, aber es fehlen Randfall-Tests für `add_window()` und Fokus-Integrität nach mehreren Add/Remove-Operationen.

**Vorschlag für Task:**
- Neue Tests hinzufügen:
  - `add_window_sets_focus_to_new_last_window`
  - `remove_focused_first_window_keeps_focus_in_bounds`
  - `sequence_add_remove_never_leaves_focused_out_of_bounds`
- Ziel: Invarianten absichern (`focused < windows.len()` oder `focused == 0` bei leerer Liste).
