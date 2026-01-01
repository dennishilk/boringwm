# 🪟 BoringWM   

![Language](https://img.shields.io/badge/language-Rust-orange)
![X11](https://img.shields.io/badge/display-X11-blue)
![Status](https://img.shields.io/badge/status-early%20development-yellow)
![Target](https://img.shields.io/badge/target-Debian%20Stable-green)
![License](https://img.shields.io/badge/license-MIT-lightgrey)

---

## 🇬🇧 🇩🇪

**BoringWM** is a minimalist, Rust-based X11 window manager.

It is intentionally **boring by design**:
predictable behavior, minimal features, no magic, no surprises.

> **BoringWM is developed on NixOS, but targets Debian Stable first.**

---

### 🎯 Project Goals

- Stability over features
- Predictable and explicit behavior
- Minimal surface area
- Long-term maintainability
- X11-first design with a possible Wayland future

BoringWM does **not** aim to be the most configurable or flashy window manager.  
It aims to be **correct**, **boring**, and **reliable**.

---

### 🧪 Development vs Target Platforms

#### Development Platform
- **NixOS**  
  Used for reproducible builds, clean Rust toolchains, and safe iteration.

#### Target / Supported Systems
- **Primary target:** Debian Stable (currently Debian 13)
- **Also expected to work on:**  
  Ubuntu, Linux Mint, Arch Linux, and similar distributions

If it runs correctly on **Debian Stable**, it is expected to run correctly
on most other Linux systems.

---

### 🧠 Design Philosophy

- **X11 only (for now)**  
  X11 provides a stable and predictable foundation.

- No scripting language in the core
- No hidden background services
- No runtime configuration magic

Configuration and extensibility may be added later,
but never at the cost of simplicity or correctness.

---

### 🚧 Project Status

BoringWM is in **early development**.

Expect:
- breaking changes
- missing features
- rough edges

The foundation comes first.

---

## 🇩🇪 

**BoringWM** ist ein minimalistischer, in Rust geschriebener X11-Window-Manager.

Er ist bewusst **boring by design**:
vorhersehbares Verhalten, minimale Features, keine Magie, keine Überraschungen.

> **BoringWM wird auf NixOS entwickelt, zielt aber primär auf Debian Stable ab.**

---

### 🎯 Projektziele

- Stabilität statt Feature-Overkill
- Vorhersehbares und explizites Verhalten
- Möglichst kleine Angriffs- und Fehlerfläche
- Langfristige Wartbarkeit
- X11-first-Design mit optionaler Wayland-Perspektive

BoringWM möchte **nicht** der konfigurierbarste oder spektakulärste
Window-Manager sein.  
Er soll **korrekt**, **langweilig** und **zuverlässig** sein.

---

### 🧪 Entwicklungs- vs. Zielplattformen

#### Entwicklungsplattform
- **NixOS**  
  Für reproduzierbare Builds, saubere Rust-Toolchains und sichere Iteration.

#### Ziel- / Unterstützte Systeme
- **Primäres Ziel:** Debian Stable (aktuell Debian 13)
- **Erwartet lauffähig auf:**  
  Ubuntu, Linux Mint, Arch Linux und vergleichbaren Distributionen

Wenn BoringWM auf **Debian Stable** korrekt läuft,  
sollte er auf den meisten anderen Linux-Systemen ebenfalls funktionieren.

---

### 🧠 Design-Philosophie

- **X11 only (vorerst)**  
  X11 bietet eine stabile und gut verstandene Basis.

- Keine Skriptsprache im Core
- Keine versteckten Hintergrunddienste
- Keine Laufzeit-Konfigurationsmagie

Konfiguration und Erweiterbarkeit können später folgen,
aber niemals auf Kosten von Einfachheit oder Korrektheit.

---

### 🚧 Projektstatus

BoringWM befindet sich in einem **frühen Entwicklungsstadium**.

Zu erwarten sind:
- Breaking Changes
- Fehlende Features
- Raue Kanten

Das Fundament hat Vorrang.
