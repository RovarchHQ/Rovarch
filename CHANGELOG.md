# Changelog

All notable changes to Rovarch will be documented in this file.

## [0.5.0] - 2026-11-26

### Changed
- Renamed the application ID from `io.github.dratiux.Rovarch` to `io.rovarch.app`.
  Desktop shortcuts, the icon, the place-file MIME package and the default-app
  associations a previous version installed are moved over on the first start;
  existing Roblox sign-in and settings are untouched.

## [0.4.0] - 2026-11-26

### Changed
- Moved `Rovarch` and `rovarch-core` to the `RovarchHQ` organization.
- Bundled Rosemary is now pinned to `RovarchHQ/rovarch-core` `v0.5.0-rovarch.1`.
- VR host-bridge glue links against the `rosemary_host_*` exports of the pinned build.

## [0.3.0] - 2026-11-26

### Changed
- Fixed `Chile` region entry showing an extra blank line by trimming region fields from RoValra API and cache.
- Wired `Test Latency` button in Servers page to actively refresh regions and display measured response latency.
- Removed the telemetry bar (`RUNTIME`, `RENDERER`, `PREFIX`, `SESSION TIME`) from the Rovarch Play page.
- Unified the crimson/red color palette and improved `Play` / `Open` button hover contrast and illumination.
- Decoupled Rovarch branding, `About` window, `README.md`, and legacy migration code from Quadra, linking to `Rovarch CORE` (`dratiux/rovarch-core`).
- Updated Rosemary release checks to follow `dratiux/rovarch-core` `-rovarch.` builds.
- Translated remaining dropdown choices (`Choice::label`) and status strings to Arabic.
- Updated `release.yml` workflow to include commit bodies in automated release notes.

## [0.2.0] - 2026-11-26

### Changed
- Removed `Rovarch Logs` from the sidebar (logs remain accessible from `Show Logs` on the Home page).
- Scoped download and installation progress text to the active client page (`Rovarch Play` or `Rovarch Studio`).
- Changed the Rovarch Studio launch button label from `Play` to `Open`.
- Replaced stretched spin controls with a compact, RTL-friendly numeric stepper for CPU cores, video memory, and VR render scale.
- Unified the crimson/red color palette and hover states across the UI.
- Completed Arabic translations across settings, dialogs, and options.

## [0.1.0-beta] - 2026-11-25

### Added
- Initial public beta release of Rovarch.
- Unified sidebar navigation for Home (`Rovarch Play` / `Rovarch Studio`), Servers, Settings, and About.
- GPU-rendered hero glow behind client icons.
- Live settings search and single-column settings layout.
- Floating Logs and About windows.
- AppImage auto-update support and CI release pipeline.
