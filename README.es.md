<p align="center">
  <img src="public/images/logo-brand.png" width="360" alt="AgentDock">
</p>

<p align="center">
  Panel de gestión OpenClaw & Hermes Agent con Asistente IA integrado — Gestión multi-motor de frameworks IA
</p>

<p align="center">
  <a href="README.md">🇨🇳 中文</a> | <a href="README.en.md">🇺🇸 English</a> | <a href="README.zh-TW.md">🇹🇼 繁體中文</a> | <a href="README.ja.md">🇯🇵 日本語</a> | <a href="README.ko.md">🇰🇷 한국어</a> | <a href="README.vi.md">🇻🇳 Tiếng Việt</a> | <strong>🇪🇸 Español</strong> | <a href="README.pt.md">🇧🇷 Português</a> | <a href="README.ru.md">🇷🇺 Русский</a> | <a href="README.fr.md">🇫🇷 Français</a> | <a href="README.de.md">🇩🇪 Deutsch</a>
</p>

<p align="center">
  <a href="https://github.com/dfpo520-claw-bot/AgentDock/releases/latest">
    <img src="https://img.shields.io/github/v/release/dfpo520-claw-bot/AgentDock?style=flat-square&color=6366f1" alt="Release">
  </a>
  <a href="https://github.com/dfpo520-claw-bot/AgentDock/releases/latest">
    <img src="https://img.shields.io/github/downloads/dfpo520-claw-bot/AgentDock/total?style=flat-square&color=8b5cf6" alt="Downloads">
  </a>
</p>

---

<p align="center">
  <img src="docs/feature-showcase.gif" width="800" alt="AgentDock Showcase">
</p>

AgentDock es un panel de gestión visual que soporta múltiples frameworks de AI Agent, actualmente con soporte dual para [OpenClaw](https://github.com/1186258278/OpenClawChineseTranslation) y [Hermes Agent](https://github.com/nousresearch/hermes-agent). Cuenta con un **asistente IA inteligente integrado** que te ayuda a instalar, diagnosticar configuraciones automáticamente, solucionar problemas y corregir errores. 8 herramientas + 4 modos + Q&A interactivo — fácil de gestionar para principiantes y expertos.

> 🌐 **Sitio web**: [github.com/dfpo520-claw-bot/AgentDock](https://github.com/dfpo520-claw-bot/AgentDock/) | 📦 **Descargar**: [GitHub Releases](https://github.com/dfpo520-claw-bot/AgentDock/releases/latest)

### 🎁 DeepAi助手 AI API

> Plataforma interna de pruebas técnicas, abierta para usuarios seleccionados. Inicia sesión diariamente para obtener créditos.

<p align="center">
  <a href="https://gpt.qt.cool"><img src="https://img.shields.io/badge/🔑 DeepAi助手 AI-gpt.qt.cool-6366f1?style=for-the-badge" alt="DeepAi助手 AI"></a>
</p>

- **Créditos por inicio de sesión diario** — Inicia sesión + invita amigos para obtener créditos de prueba
- **API compatible con OpenAI** — Integración perfecta con OpenClaw
- **Política de recursos** — Límite de velocidad + límite de solicitudes, posible cola en horas pico
- **Disponibilidad de modelos** — Modelos/APIs según la página actual, posible rotación de versiones

> ⚠️ **Cumplimiento**: Solo para pruebas técnicas. Prohibido el uso ilegal o eludir mecanismos de seguridad. Mantén tu API Key segura. Las reglas están sujetas a las últimas políticas de la plataforma.

### 🔥 Soporte para placas de desarrollo / Dispositivos embebidos

- **Orange Pi / Raspberry Pi / RK3588** — `npm run serve` para ejecutar
- **Docker ARM64** — `docker run ghcr.io/DeepAi助手/openclaw:latest`
- **Armbian / Debian / Ubuntu Server** — Detección automática de arquitectura
- Sin necesidad de Rust / Tauri / GUI — **solo Node.js 18+**

## Comunidad

Una comunidad de desarrolladores y entusiastas apasionados por los AI Agents — ¡únete!

<p align="center">
  <a href="https://discord.gg/U9AttmsNHh"><strong>Discord</strong></a>
  &nbsp;·&nbsp;
  <a href="https://github.com/dfpo520-claw-bot/AgentDock/discussions"><strong>Discussions</strong></a>
  &nbsp;·&nbsp;
  <a href="https://github.com/dfpo520-claw-bot/AgentDock/issues/new"><strong>Reportar Issue</strong></a>
</p>

## Características

- **🤖 Asistente IA (Nuevo)** — Asistente IA integrado, 4 modos + 8 herramientas + Q&A interactivo
- **🧩 Arquitectura multi-motor** — Soporta OpenClaw y Hermes Agent dual, conmutación libre, gestión independiente
- **🤖 Chat Hermes Agent** — Interfaz de chat Hermes Agent integrada, visualización de llamadas a herramientas, acceso a archivos, streaming SSE
- **🖼️ Reconocimiento de imágenes** — Pega capturas o arrastra imágenes, IA analiza automáticamente
- **Panel** — Vista general del sistema, monitoreo de servicios en tiempo real
- **Gestión de servicios** — Inicio/parada de OpenClaw / Hermes Gateway, detección de versión y actualización
- **Configuración de modelos** — Gestión multi-proveedor, pruebas de conectividad por lotes, ordenar arrastrando
- **Configuración de Gateway** — Puerto, alcance de acceso, Token de autenticación, Tailscale
- **Canales de mensajería** — Gestión unificada de Telegram, Discord, Feishu, DingTalk, QQ
- **Comunicación y automatización** — Configuración de mensajes, difusión, Webhooks, aprobación de ejecución
- **Análisis de uso** — Uso de tokens, costos API, rankings de modelos/proveedores
- **Gestión de Agents** — CRUD de Agents, edición de identidad, gestión de workspace
- **Chat** — Streaming, renderizado Markdown, gestión de sesiones
- **Tareas programadas** — Ejecución programada con Cron, entrega multicanal
- **Visor de logs** — Logs en tiempo real multi-fuente y búsqueda por palabras clave
- **Gestión de memoria** — Ver/editar archivos de memoria, exportar ZIP, cambiar Agent
- **DeepAi助手 AI API** — Plataforma de pruebas interna, compatible con OpenAI
- **Herramientas de extensión** — Gestión de túneles cftunnel, monitoreo de ClawApp
- **Acerca de** — Información de versión, enlaces de comunidad, proyectos relacionados

## Descargar e instalar

Visita [Releases](https://github.com/dfpo520-claw-bot/AgentDock/releases/latest) para la última versión:

| Plataforma | Instalador |
|-----------|-----------|
| **Windows** | `.exe` (recomendado) o `.msi` |
| **macOS Apple Silicon** | `.dmg` (aarch64) |
| **macOS Intel** | `.dmg` (x64) |
| **Linux** | `.AppImage` / `.deb` / `.rpm` |

### Servidor Linux (Versión Web)

```bash
curl -fsSL https://raw.githubusercontent.com/dfpo520-claw-bot/AgentDock/main/scripts/linux-deploy.sh | bash
```

### Docker

```bash
docker run -d --name agentdock --restart unless-stopped \
  -p 1420:1420 -v agentdock-data:/root/.openclaw \
  node:22-slim \
  sh -c "apt-get update && apt-get install -y git && \
    npm install -g @DeepAi助手/openclaw-zh --registry https://registry.npmmirror.com && \
    git clone https://github.com/dfpo520-claw-bot/AgentDock.git /app && \
    cd /app && npm install && npm run build && npm run serve"
```

## Inicio rápido

1. **Configuración inicial** — Primera ejecución detecta automáticamente Node.js, Git, OpenClaw. Instalación con un clic si falta
2. **Configurar modelos** — Añadir proveedores de IA (DeepSeek, OpenAI, Ollama, etc.) y probar conectividad
3. **Iniciar Gateway** — Ir a Gestión de servicios, clic en "Iniciar". Estado verde = listo
4. **Empezar a chatear** — Ir a Chat en vivo, seleccionar modelo y comenzar conversación

## Arquitectura técnica

| Capa | Tecnología | Descripción |
|------|-----------|-------------|
| Frontend | Vanilla JS + Vite | Sin framework, ligero |
| Backend | Rust + Tauri v2 | Rendimiento nativo, multiplataforma |
| Comunicación | Tauri IPC + Shell Plugin | Puente frontend-backend |
| Estilos | Pure CSS (CSS Variables) | Temas oscuro/claro |

## Compilar desde código fuente

```bash
git clone https://github.com/dfpo520-claw-bot/AgentDock.git
cd agentdock && npm install

# Escritorio (requiere Rust + Tauri v2)
npm run tauri dev        # Desarrollo
npm run tauri build      # Producción

# Solo Web (sin Rust)
npm run dev              # Hot reload
npm run build && npm run serve  # Producción
```

## Proyectos relacionados

| Proyecto | Descripción |
|----------|-------------|
| [OpenClaw](https://github.com/1186258278/OpenClawChineseTranslation) | Framework AI Agent |
| [ClawApp](https://github.com/DeepAi助手/clawapp) | Cliente móvil multiplataforma |
| [cftunnel](https://github.com/DeepAi助手/cftunnel) | Herramienta Cloudflare Tunnel |

## Contribuir

Issues y Pull Requests son bienvenidos. Ver [CONTRIBUTING.md](CONTRIBUTING.md).


## Sponsor

If you find this project useful, consider supporting us via USDT (BNB Smart Chain):

<img src="public/images/bnbqr.jpg" alt="Sponsor QR" width="180">

```
0xbdd7ebdf2b30d873e556799711021c6671ffe88f
```

## Contact

- **Support**: [GitHub Issues](https://github.com/dfpo520-claw-bot/AgentDock/issues)
- **Website**: [github.com/dfpo520-claw-bot/AgentDock](https://github.com/dfpo520-claw-bot/AgentDock)
- **Product**: [github.com/dfpo520-claw-bot/AgentDock](https://github.com/dfpo520-claw-bot/AgentDock)
© 2026 DeepAi助手 | [github.com/dfpo520-claw-bot/AgentDock](https://github.com/dfpo520-claw-bot/AgentDock)
