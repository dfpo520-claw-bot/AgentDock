# DeepAi Assistant Settings Design

## Goal

Simplify the chat-page assistant settings modal so the API configuration defaults to the DeepAi assistant experience. The modal should present DeepAi as the only visible provider choice while preserving the existing API Key, Base URL, model, connection test, remote model fetch, save, and OpenClaw sync flows.

## Scope

This change applies only to the settings modal opened from the chat page in `src/pages/assistant.js`.

In scope:

- Remove visible multi-provider preset choices from the assistant API settings tab.
- Show a single DeepAi assistant provider surface in the API settings tab.
- Default `API Base URL` to `https://api.deepai.wang/v1` when no saved value exists.
- Add a `获取 API Key` link beside the `API Base URL` label that opens `https://api.deepai.wang/`.
- Keep `API Key`, `模型`, connection test, model fetch, save, and config sync behavior working.

Out of scope:

- No changes to the system settings page.
- No backend API changes.
- No config schema migration.
- No removal of tools, persona, or knowledge tabs.
- No change to existing saved custom Base URL values.

## Current Behavior

The assistant settings modal currently renders a generic API configuration area with multiple provider presets and provider-specific shortcuts. The modal already supports:

- Base URL input
- API Key input
- API type selection
- Model input and model dropdown
- Connection testing
- Remote model listing
- Sync to and from OpenClaw config

The current problem is the first-run API surface still feels like a general provider picker instead of a product-owned DeepAi assistant configuration flow.

## Selected Approach

Use a display-layer refactor only.

- Keep the existing config object, save handler, testing flow, model fetch flow, and sync flow unchanged unless they need a small compatibility tweak for the default Base URL.
- Replace the visible provider preset button group with a single DeepAi assistant provider banner/chip.
- Seed the Base URL field from a product default constant when the saved config does not already provide a value.

This approach keeps the behavior stable while changing the default visual entry point.

## UX Details

### API tab

- The tab remains the default tab in the modal.
- The top of the API section shows `DeepAi助手` as the only visible provider.
- The previous provider preset strip is removed from the UI.

### Base URL row

- Label: `API Base URL`
- Right-side inline action: `获取 API Key`
- Link target: `https://api.deepai.wang/`
- Open in a new browser tab/window with safe external-link attributes.

### Base URL defaulting

- If `c.baseUrl` is empty when the modal opens, the input renders with `https://api.deepai.wang/v1`.
- If `c.baseUrl` already has a saved value, render that value instead.
- Saving should continue to persist the actual current input value.

## Testing Strategy

Add a regression test that verifies:

- The assistant settings template no longer renders the provider preset strip.
- The settings template still renders the API configuration controls.
- `DeepAi助手` appears as the visible provider surface.
- The default Base URL string `https://api.deepai.wang/v1` is present in the settings implementation.
- The modal includes a link to `https://api.deepai.wang/`.

Then run focused frontend tests and a production build.
