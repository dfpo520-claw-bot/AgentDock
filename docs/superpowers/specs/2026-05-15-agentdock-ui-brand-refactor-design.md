# AgentDock UI And Brand Refactor Design

## Goal

Refactor the AgentDock frontend into a production-grade desktop operations console while preserving all existing functionality, route behavior, Tauri IPC contracts, runtime integrations, and backend command behavior.

This phase changes UI, layout, visual system, visible brand assets, visible product copy, and release entry points. It must not change feature semantics.

## Selected Direction

The selected direction is **A: Modern Ops Console**.

Reference systems:

- [shadcn/ui Blocks](https://ui.shadcn.com/blocks): primary reference for calm admin layout, subtle cards, spacing, page headers, and route-level composition.
- [Radix UI](https://www.radix-ui.com/): accessibility and interaction reference for focus states, menus, dialogs, popovers, and keyboard behavior.
- [Ant Design Pro](https://github.com/ant-design/ant-design-pro): secondary reference for dense enterprise tables, filters, forms, and operational workflows.
- [Mantine](https://mantine.dev/): secondary reference for polished component proportions and readable form density.

AgentDock should not migrate to React or directly import those component libraries in this phase. The project stays on Vite + vanilla JavaScript + CSS. We borrow their interaction and layout patterns, then implement them with the existing app architecture.

## Non-Goals

- No backend command rewrite.
- No route removal.
- No data model changes.
- No change to Tauri command names or IPC payloads.
- No migration from vanilla JavaScript to React, Vue, Svelte, or another framework.
- No full rewrite of OpenClaw or Hermes pages in one step.
- No forced rename of compatibility paths such as `agentdock.json` when they are required for migration or legacy aliases.
- No license or upstream fork strategy changes in this phase.

## Current Frontend State

The current frontend is a Vite app with:

- Global shell in `src/main.js`.
- Hash routing in `src/router.js`.
- Sidebar and global controls in `src/components/sidebar.js`.
- Shared UI primitives in `src/components/modal.js`, `src/components/toast.js`, `src/components/ai-drawer.js`, and related files.
- Global CSS in `src/style/variables.css`, `src/style/layout.css`, `src/style/components.css`, `src/style/pages.css`, `src/style/chat.css`, `src/style/assistant.css`, and feature-specific CSS files.
- Large route modules under `src/pages`.
- Engine-specific route modules and CSS under `src/engines/hermes` and `src/engines/openclaw`.
- Product constants in `src/lib/product-identity.js` and `src/lib/product-config.js`.
- i18n dictionaries under `src/locales/modules` and generated locale JSON files.

The current UI already contains most product functionality. The main problem is inconsistent visual hierarchy, inherited brand copy, mixed component styles, and scattered AgentDock/DeepAi助手 strings.

## Product Identity

Primary product identity:

- Product name: `AgentDock`
- Product id: `agentdock`
- Assistant name: `DeepAi助手`
- English assistant display name: `DeepAi Assistant`
- Tagline: `Multi-engine AI agent operations console`
- Desktop positioning: production desktop console for multi-engine AI agent operations.

Visible product copy should use AgentDock language:

- "AgentDock" for the app shell, About page, page titles, release notes, installer metadata, and visible product references.
- "DeepAi助手" for the built-in AI assistant in Chinese UI.
- "DeepAi Assistant" for English UI.
- "OpenClaw" and "Hermes" remain engine/runtime names.

The legacy product name `AgentDock` remains only when explaining compatibility, migration, or upstream reference context.

## Brand Asset System

The final brand asset set should include:

- Primary logo: AgentDock wordmark plus compact AD monogram.
- App icon: square icon with an AD monogram and a dock/ring motif.
- Sidebar logo: compact monogram that remains readable at 28px.
- Tray icon: simplified high-contrast mark that is readable at small sizes.
- Favicon: generated from the app icon.
- Splash/loading mark: same monogram, no unrelated illustration.
- README and docs logo: horizontal logo on light background.
- Release cover image: simple product card showing AgentDock name, icon, and version.

Visual direction:

- Primary palette: deep neutral blue-gray surfaces with teal operational accent.
- Secondary accent: restrained blue for links and release/update actions.
- Status colors: green success, amber warning, red destructive/error, blue informational.
- Avoid a one-note purple/blue gradient system.
- Avoid beige/cream/sand palettes.
- Avoid decorative blobs, orbs, and marketing-style hero art inside the desktop app.

Logo concept:

- Use an `AD` monogram inside a dock/ring container.
- Shape language: square with 8px-radius equivalent, stable and icon-friendly.
- The icon should work in light mode, dark mode, installer assets, and Windows taskbar contexts.

## UI Architecture

The refactor should build a shared shell and component system without breaking route code.

Target structure:

```text
src/style/
  variables.css       global tokens
  reset.css           browser normalization
  layout.css          app shell, sidebar, content regions
  components.css      shared primitives
  pages.css           common page layouts
  chat.css            chat route specifics
  assistant.css       DeepAi assistant specifics

src/components/
  sidebar.js          navigation and product shell controls
  modal.js            dialogs and confirmations
  toast.js            notifications
  ai-drawer.js        DeepAi global entry
  kernel-badge.js     runtime/version badge
  cli-conflict-banner.js
```

This phase should prefer CSS token and component class updates over deep page logic rewrites. If a page needs markup adjustment for layout consistency, the markup change must preserve existing DOM event bindings and data flow.

## Layout System

The desktop shell should become a quiet operations console:

- Persistent left sidebar with compact product identity, engine switcher, grouped navigation, and utility footer.
- Main content area with a consistent page header and optional route-level toolbar.
- Top or inline status surfaces for Gateway ownership, update prompts, and security warnings.
- Content width rules:
  - Operational pages can use full available width.
  - Form-heavy pages should use a constrained inner width.
  - Chat and assistant pages can use full-height split layouts.
- Cards should be used for repeated items, panels, modals, and bounded tools.
- Page sections should be unframed layouts or full-width bands, not nested cards.

Core route layout targets:

- Dashboard: status summary cards, runtime health, quick actions, and recent operational signals.
- Services: dense control table plus action panels; destructive or service-changing actions stay visually clear.
- Settings: grouped forms with save/test controls, readable descriptions, and clear dirty/saved states.
- Assistant: DeepAi助手 as a professional split workspace with session list, messages, tool policy controls, and confirmation surfaces.
- Logs: source selector plus log viewer with redaction status and export action.
- About: product identity, version, update status, links, release channel, and referenced open-source projects.

Secondary routes should adopt the same page header, toolbar, table, empty state, and card primitives after the core route patterns are stable.

## Component System

The shared component language should include:

- `page-shell`: consistent route padding, max-width behavior, and route-level overflow handling.
- `page-header`: title, subtitle, status badges, primary/secondary actions.
- `button`: primary, secondary, ghost, destructive, icon-only, loading.
- `icon-button`: fixed square controls with tooltip/title support.
- `badge` and `status-pill`: small state labels for runtime, release, gateway, and model status.
- `stat-card`: compact operational metrics.
- `data-table`: dense rows, sticky-ish header where practical, row actions, empty/error/loading states.
- `form-field`: label, description, input, help text, error text.
- `segmented-control`: modes such as read-only/plan/unlimited and log source filters.
- `toolbar`: filter/search/action row.
- `modal`: confirmation and content dialogs with consistent destructive states.
- `toast`: consistent success/error/info/warning notifications.
- `empty-state`: quiet placeholder for missing data.
- `code-block`: command snippets, paths, and logs.

Existing classes can remain during migration, but new or updated pages should converge on these shared primitives.

## Interaction Rules

The refactor must preserve safety behavior:

- Assistant dangerous tool confirmation policy remains intact.
- Service start/stop/restart/upgrade/uninstall actions keep confirmation and error surfaces.
- Settings writes keep existing command calls and validation behavior.
- Gateway ownership guidance remains visible and actionable.
- Logs and exports continue to redact secrets.
- Update checks keep current rollback and manifest behavior.

Interaction quality expectations:

- Keyboard focus must be visible.
- Icon-only buttons must have accessible labels via `title`, `aria-label`, or visible adjacent text.
- Disabled/loading states must not shift layout.
- Mobile/narrow desktop views must not overlap text or controls.
- Text should not scale with viewport width.
- Letter spacing should remain zero for normal UI text; uppercase labels may use small positive tracking only where already established.

## Brand And Copy Replacement Rules

Replace visible product copy:

- `AgentDock` -> `AgentDock`
- `agentdock` -> `agentdock` when it is visible product copy, release link text, public docs for new users, service worker notification tags, or generated asset names.
- `DeepAiepAi助手` -> `DeepAi助手`
- English assistant labels should use `DeepAi Assistant`.

Do not blindly replace compatibility strings:

- `agentdock.json` remains as a legacy config filename where migration support depends on it.
- `agentdock_authed`, `agentdock_must_change_pw`, and similar storage keys may remain until a deliberate storage migration task is planned.
- `.disabled-by-agentdock-...` quarantine markers remain unless a backend migration strategy is created.
- Existing comments may be cleaned when touched, but comments alone are not release blockers.
- Upstream reference docs may mention AgentDock when explicitly describing the fork baseline or referenced project.

Replacement priority:

1. User-visible app UI and locale strings.
2. About page and release/update entry points.
3. Public assets and screenshots.
4. Browser notification titles and tags.
5. README and user-facing docs.
6. Internal comments only when already editing nearby code.

## Release Entry Points

All user-facing release and support links should point to AgentDock-owned destinations:

- Repository: `https://github.com/dfpo520-claw-bot/AgentDock`
- Releases: `https://github.com/dfpo520-claw-bot/AgentDock/releases`
- Issues/support: `https://github.com/dfpo520-claw-bot/AgentDock/issues`
- Update manifest: `https://raw.githubusercontent.com/dfpo520-claw-bot/AgentDock/master/update/latest.json`

The About page should show AgentDock-owned release status and still disclose referenced open-source projects in a separate attribution section.

## Implementation Strategy

The implementation should be staged to keep the app runnable after each task.

### Stage 1: Brand And Tokens

- Update `src/lib/product-identity.js` if needed.
- Replace core visible strings in `src/locales/modules/sidebar.js`, `src/locales/modules/setup.js`, `src/locales/modules/about.js`, `src/locales/modules/assistant.js`, and any other modules found by search.
- Replace service worker visible notification title/tag in `public/push-sw.js`.
- Introduce refreshed tokens in `src/style/variables.css`.
- Keep compatibility keys and file paths unchanged unless explicitly scoped.

### Stage 2: App Shell And Shared Components

- Refactor `src/style/layout.css` for the new sidebar and main content frame.
- Refactor `src/style/components.css` for buttons, cards, status pills, forms, tables, modals, and toasts.
- Update `src/components/sidebar.js` to use the final logo/brand shell and cleaner nav density.
- Keep route definitions and engine switching behavior unchanged.

### Stage 3: Core Route Layouts

Apply the new shell and primitives first to:

- `src/pages/dashboard.js`
- `src/pages/services.js`
- `src/pages/settings.js`
- `src/pages/assistant.js`
- `src/pages/logs.js`
- `src/pages/about.js`

These routes become the visual reference for the rest of the app.

### Stage 4: Brand Assets

- Generate or replace source logo artwork.
- Regenerate Tauri icons under `src-tauri/icons`.
- Sync `public/favicon.ico`.
- Replace visible images under `public/images` and `docs` when they are product-owned assets.
- Update README logo and release cover image.

### Stage 5: Secondary Routes And Cleanup

Roll shared layout and components across the remaining OpenClaw and Hermes routes:

- Models, agents, channels, communication, notifications, security.
- Memory, dreaming, cron, usage, skills, plugin hub.
- Hermes dashboard, services, chat, logs, setup, skills, sessions, and extension routes.

This stage should be split into smaller implementation tasks because several files are large.

## Testing And Verification

Every implementation batch should run:

```powershell
node --test tests\*.test.js
cargo check --manifest-path src-tauri\Cargo.toml
node node_modules\vite\bin\vite.js build
```

Route smoke should cover at least:

- `/dashboard`
- `/settings`
- `/services`
- `/assistant`
- `/logs`
- `/about`

Manual desktop checks should verify:

- Desktop launch is not blank.
- Sidebar collapse/expand works.
- Language switcher works.
- Theme switcher works.
- Gateway banner remains functional.
- Assistant mode switching and dangerous tool confirmation remain functional.
- Settings write and test actions still call the original commands.
- Logs and exported content remain redacted.
- Window sizes do not cause text/control overlap.

Screenshot verification should include:

- Desktop width around 1440px.
- Narrow desktop/tablet width around 900px.
- Mobile-like narrow width around 390px for Web mode and responsive sanity.

## Success Criteria

- The selected A Modern Ops Console direction is visible across the app shell and core routes.
- All existing core functionality remains reachable.
- No visible `AgentDock` remains in production app UI except explicit legacy/upstream references.
- No visible `DeepAiepAi助手` remains; the assistant is shown as `DeepAi助手`.
- Product-owned assets and release links point to AgentDock.
- Build and test commands pass.
- UI smoke routes pass.
- Existing signing and release handoff docs remain accurate.

## Risks

- The current frontend has very large route files, especially Assistant, Chat, Channels, and Hermes routes. Visual changes should be staged and verified route by route.
- Locale files are large and generated JSON files may need synchronized updates after module edits.
- Blind string replacement can break compatibility paths or migration logic.
- Regenerating icons may create many binary changes; icon work should be isolated in its own commit.
- The existing `i18n` bundle size warning remains a known issue and should not be mixed with the visual refactor unless a dedicated lazy-loading task is created.

## Recommended Next Step

After this design is reviewed, write an implementation plan that starts with brand/string replacement and token-level UI foundations, then moves through shell, core routes, assets, and secondary routes in separate commits.
