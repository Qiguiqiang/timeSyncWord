# Changelog

All notable changes to OpenTimeSync are documented in this file.

## [2.1.11] - 2026-06-08

### Changed
- Replaced remote Google font loading with local system font stacks so the app typography stays consistent across Windows, macOS, Linux, and offline installs.
- Reduced the main clock and stat number size ceilings to keep the time display stable after upgrades.

### Fixed
- Fixed an updater self-deadlock when starting a download, which could freeze the app before progress polling began.
- Improved updater modal state handling so download/install buttons and progress text update immediately after the user starts an update.

## [2.1.10] - 2026-06-08

### Fixed
- Fixed macOS Dock reopen behavior after disabling the floating widget from its `X` button, so a hidden main window is restored instead of leaving the app running with no usable surface.
- Fixed floating widget close-button hit testing so clicking the close control cannot be misread as widget drag or restore.

## [2.1.9] - 2026-06-08

### Fixed
- Fixed floating widget drag snap-back by removing main-window focus-driven visibility refreshes that were reapplying stale widget coordinates during drag operations.

## [2.1.8] - 2026-06-08

### Fixed
- Fixed floating widget movement by switching back to a more reliable direct native drag start path for the compact widget window.
- Fixed floating widget z-order so the widget window is always created as top-most and is no longer easily covered by normal application windows after it is shown.

## [2.1.7] - 2026-06-08

### Fixed
- Fixed floating widget content scaling so the live widget now scales its precision badge, time text, and close control using the actual widget window size, eliminating stale inner sizing when the scale percentage changes while the widget is already open.

## [2.1.6] - 2026-06-08

### Fixed
- Fixed floating widget content scaling so the precision badge, time text, and close control now follow the configured widget scale percentage in the actual widget window instead of only resizing the outer host window.

## [2.1.5] - 2026-06-08

### Fixed
- Fixed the tray widget menu interaction by keeping the tray menu stable instead of rebuilding it during visibility refreshes, so the right-click menu remains hoverable and clickable.
- Fixed the tray-driven widget toggle path by introducing an explicit manual widget visibility state, so showing the widget from the tray no longer gets immediately cancelled by the normal minimize-only visibility rule.

## [2.1.4] - 2026-06-08

### Changed
- Enabled the macOS private API path so startup splash and floating widget windows can use true transparent overlays on macOS outside the App Store distribution path.

### Fixed
- Fixed transparent startup and widget overlays after the earlier macOS compatibility patch by restoring platform-appropriate transparent window creation for macOS as well as Windows.
- Added a tray menu action to show or hide the floating widget directly from the tray icon right-click menu.

## [2.1.3] - 2026-06-08

### Fixed
- Fixed the startup splash window so transparent areas remain truly transparent again on Windows instead of falling back to an opaque rectangle after the macOS build compatibility change.
- Fixed the floating widget window so non-drawn areas return to transparent behavior again instead of showing an opaque host window.
- Added a tray menu action to show or hide the floating widget directly from the tray icon right-click menu, without forcing users back through settings first.

## [2.1.2] - 2026-06-07

### Changed
- Changed the floating widget into a safer third-party overlay model that stays inside the monitor work area so it no longer covers Windows tray controls.
- Changed widget size control from a delayed numeric input into a real-time percentage slider and kept the final size persisted across restarts.

### Fixed
- Fixed floating widget drag by routing it through a dedicated native widget drag command instead of the main-window drag path.
- Fixed widget position persistence by saving from widget window move events, so dragged positions survive hide, restore, and restart.
- Fixed widget close behavior so the `X` button fully disables the widget setting without freezing and the close button remains fully visible inside the widget bounds.
- Fixed macOS build compatibility by removing unsupported builder transparency calls so both Intel and Apple Silicon DMG pipelines pass in GitHub Actions.

## [2.1.1] - 2026-06-07

### Fixed
- Fixed the floating widget settings freeze by removing a self-deadlock in config persistence and adding backward-compatible default loading for old configs that did not yet contain widget scale.

## [2.1.0] - 2026-06-07

### Added
- Added widget size control to settings for the floating gadget.

### Fixed
- Fixed the floating widget interaction chain further so close disables widget mode cleanly, drag saves the final position, and the compact floating display keeps rendering after state changes.

## [2.0.99] - 2026-06-07

### Added
- Added widget size control in settings so the floating gadget can be enlarged or reduced without code changes.

### Fixed
- Fixed the floating widget interaction chain so close updates the enabled state cleanly and drag now persists the final position through the native drag flow.

## [2.0.98] - 2026-06-07

### Fixed
- Fixed floating widget interaction so drag uses native window dragging, double click restores the main window, close actually disables the widget, and the final dragged position can be persisted reliably.

## [2.0.97] - 2026-06-07

### Changed
- Changed the widget back from the Windows AppBar experiment to a true floating desktop gadget window that does not reserve taskbar area.
- Added drag-to-move, hover-to-show-close-button, and persisted floating position for the widget.

### Fixed
- Fixed the widget mode so closing the floating gadget dismisses it cleanly instead of leaving tray-collapse state behind.

## [2.0.96] - 2026-06-07

### Changed
- Changed the Windows widget implementation over to the new AppBar-based state machine and reserved edge strip, replacing the prior floating tray-following window path.

## [2.0.95] - 2026-06-07

### Fixed
- Fixed the compact widget frontend after removing extra fields so it no longer references deleted DOM nodes and can keep rendering reliably across repeated minimize and restore cycles.

## [2.0.94] - 2026-06-07

### Changed
- Changed widget placement to use a fixed taskbar-safe reserve zone instead of following the tray icon rectangle, preventing the strip from covering the "show hidden icons" button and nearby system tray controls.

## [2.0.93] - 2026-06-07

### Changed
- Changed the widget into a smaller read-only strip that shows only precision tier and current time.
- Changed widget positioning to reserve more space from the right-side system tray cluster so it does not visually cover core system icons as aggressively.

### Fixed
- Fixed the widget reveal path by explicitly showing it during hide-to-tray transitions instead of relying only on later visibility polling.

## [2.0.92] - 2026-06-07

### Fixed
- Fixed the widget visibility state machine so tray-collapse mode no longer depends only on window minimize visibility callbacks and can explicitly show the widget after hide-to-tray transitions.

## [2.0.91] - 2026-06-07

### Fixed
- Fixed the widget refresh path so the app now continuously re-evaluates tray and widget visibility while running, instead of relying only on a few window events that could be missed by some minimize flows.

## [2.0.90] - 2026-06-07

### Fixed
- Fixed the widget visibility regression where the strip window could be created and positioned but remain effectively invisible because it was forced to the bottom window layer.

## [2.0.89] - 2026-06-07

### Changed
- Changed startup wordmark styling again to reduce size, bloom, and stroke intensity so the transparent splash no longer looks overexposed.

### Fixed
- Fixed the NTP validation bug that rejected legitimate server timestamps whenever the local machine clock was already off by several seconds, which made the sync path appear completely broken.

## [2.0.88] - 2026-06-07

### Changed
- Changed the startup logo rendering again to cut glow intensity and reduce the overexposed look on transparent startup.
- Changed startup fallback wiring so the splash screen always yields to the main interface after the fixed upper bound even if background sync is still slow.

### Fixed
- Fixed the missing startup failsafe wiring that still allowed some runs to stay on the splash screen longer than intended.

## [2.0.87] - 2026-06-07

### Changed
- Changed splashscreen creation to be manual so the runtime transparent background, click-through, and compact sizing rules can actually take effect.
- Changed the startup typography and spacing again to reduce bloom and overexposure and keep the `OpenTimeSync` mark readable on transparent startup.

### Fixed
- Fixed the startup regression where the splash window could still appear as a large bright rectangle because the auto-created config window bypassed the runtime transparent settings.

## [2.0.86] - 2026-06-07

### Changed
- Changed startup completion rules so the app can enter the main interface as soon as it has a usable first sync result instead of waiting for stricter calibration stages.
- Changed the settings panel to become vertically scrollable when the window height is insufficient.

### Fixed
- Fixed the startup hang where the splash screen could remain visible even after a usable sync result existed.
- Fixed the transparent startup window input issue by making the splash window ignore cursor events so desktop interactions can pass through outside the rendered mark itself.
- Fixed the settings usability issue where content below the fold could not be reached when the main window was not maximized.

## [2.0.85] - 2026-06-07

### Changed
- Changed the startup logo from a soft three-line gradient wordmark to a sharper single-line `OpenTimeSync` mark so it remains readable on transparent startup.
- Changed widget placement again so it first snaps to the actual taskbar reserved area instead of falling back into the normal desktop workspace.

### Fixed
- Fixed the boot logo readability issue where the large startup lettering still looked too soft and unclear.
- Fixed another widget placement regression where the strip could still drift back into the desktop area instead of staying aligned with the taskbar band.

## [2.0.84] - 2026-06-07

### Changed
- Changed startup rendering to use a dedicated transparent splash window instead of drawing the boot scene inside the main application window.
- Changed the main window to stay hidden during first calibration so only the `OpenTimeSync` wordmark and loading description remain visible at boot.

### Fixed
- Fixed the startup transparency issue where the user could still see the semi-transparent main application silhouette behind the boot scene.

## [2.0.83] - 2026-06-07

### Changed
- Changed widget behavior so the edge widget no longer stays forced above other application windows and instead uses a less intrusive system-edge presentation.
- Changed the widget switch semantics so enabling it now means the app collapses to the system tray on minimize or close, with the edge widget acting as an auxiliary surface instead of the primary restore path.

### Added
- Added a real system tray entry with restore and exit actions, plus left-click restore behavior for the collapsed app state.

### Fixed
- Fixed the close and minimize flow so the main window can hide to tray instead of exiting when the widget/tray mode is enabled.
- Fixed version discipline by moving the local app, package, and Tauri bundle metadata together to `2.0.83`.

## [2.0.82] - 2026-06-07

### Changed
- Changed the boot scene again so the startup stage hides the full main interface entirely and keeps only the large animated `OPEN TIME SYNC` wordmark plus a thin status line.
- Changed the boot wordmark styling to stay visually solid and bright instead of inheriting the translucent feel of the transparent startup window.

### Fixed
- Fixed the startup issue where a semi-transparent main-window silhouette was still visible behind the boot scene.
- Fixed the startup flow so it no longer exits the boot scene merely because an early sample arrived before full stabilization.

## [2.0.81] - 2026-06-07

### Changed
- Changed the startup presentation from a boxed overlay into a large transparent animated `OPEN TIME SYNC` wordmark so the desktop remains visible during boot.
- Changed boot completion rules so the main interface waits for a stable first calibration result instead of revealing after an early partial sample.
- Changed the widget presentation from a floating desktop card into a thin taskbar-edge strip aligned to the reserved taskbar area.

### Fixed
- Fixed the boot overlay exiting too early before the first calibration had actually stabilized.
- Fixed the widget placement logic so it no longer drops into the normal desktop workspace area by default.

## [2.0.80] - 2026-06-07

### Added
- Added a cinematic cold-start boot overlay with animated `OPEN TIME SYNC` wordmark, live startup status text, and progress feedback while the first synchronization is happening in the background.

### Changed
- Changed the main window to support a transparent startup presentation so the desktop can show through the animated boot screen before the core UI fades in.

## [2.0.79] - 2026-06-07

### Added
- Added LAN sync roles so one device can act as the LAN master and other devices can follow that calibrated time as LAN slaves.
- Added a minimize-triggered desktop widget window that shows synchronized time, precision color, and hover details, and can restore the main window on click.
- Added update progress UI with version notes, download progress, and clearer install/restart messaging.
- Added persistent sync settings for mode, LAN host, pair code, widget switch, sync interval, and selected NTP server.

### Changed
- Changed first-launch behavior to show local system time first, then enter calibration only when auto sync is enabled.
- Changed calibration behavior to use a 2-second fast interval during startup calibration and fall back to the configured cadence after stabilization or timeout.
- Changed updater metadata flow so app-side version notes can come from changelog-backed release notes instead of a generic placeholder string.

### Fixed
- Fixed network-no-data handling so offset and precision no longer disappear when a refresh lacks a fresh RTT sample.
- Fixed the sync interval bug where the UI could show a custom value but the backend still kept the old default cadence.
- Fixed version-discipline mismatches by preparing a single version source path across app UI, build metadata, and release packaging.

## [2.0.78] - 2026-06-07

### Added
- Added settings tooltips for immediate NTP sync, immediate system sync, auto sync, sync interval, and version display.
- Added update modal groundwork with release-note display support.

### Changed
- Changed default auto sync to enabled and the default cadence to 5 seconds.
- Changed the minimum sync interval from 5 seconds to 2 seconds.

### Fixed
- Fixed startup behavior so the app can show local time before the first usable NTP sample arrives.
- Fixed update check messaging so available versions are surfaced more clearly in the UI.
