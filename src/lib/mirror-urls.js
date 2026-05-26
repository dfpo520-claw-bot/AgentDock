/**
 * GitHub URL 管理
 * 当前公开仓库暂未配置独立镜像，无法访问 GitHub 时仍返回官方仓库地址。
 */

const GITHUB_ORG = 'https://github.com/dfpo520-claw-bot'
const GITEE_ORG = GITHUB_ORG
const GITHUB_RAW = 'https://raw.githubusercontent.com/dfpo520-claw-bot'
const GITEE_RAW = GITHUB_RAW

// 仓库名映射（预留给后续镜像仓库，名称不同时需映射）
const REPO_MAP = {
  agentdock: 'AgentDock',
  clawapp: 'clawapp',
  cftunnel: 'cftunnel',
  'openclaw-zh': 'openclaw-zh',
}

/**
 * 探测 GitHub 是否可达（3s 超时）
 * 结果缓存 5 分钟
 */
let _githubReachable = null
let _lastCheck = 0
const CHECK_TTL = 300000 // 5min

async function isGithubReachable() {
  const now = Date.now()
  if (_githubReachable !== null && now - _lastCheck < CHECK_TTL) return _githubReachable
  try {
    const ctrl = new AbortController()
    const timer = setTimeout(() => ctrl.abort(), 3000)
    await fetch('https://github.com/favicon.ico', { method: 'HEAD', mode: 'no-cors', signal: ctrl.signal })
    clearTimeout(timer)
    _githubReachable = true
  } catch {
    _githubReachable = false
  }
  _lastCheck = now
  return _githubReachable
}

/**
 * 获取仓库 URL（优先 GitHub，暂未配置镜像时仍返回 GitHub）
 * @param {string} repo - 仓库名，如 'agentdock'
 * @param {string} [path] - 可选路径，如 '/releases'、'/issues/new'
 */
export async function repoUrl(repo, path = '') {
  const mappedRepo = REPO_MAP[repo] || repo
  if (await isGithubReachable()) {
    return `${GITHUB_ORG}/${mappedRepo}${path}`
  }
  return `${GITEE_ORG}/${mappedRepo}${path}`
}

/**
 * 同步版本：同时返回主仓库和镜像 URL，让 UI 可以展示两个链接
 * @param {string} repo
 * @param {string} [path]
 */
export function repoBothUrls(repo, path = '') {
  const mappedRepo = REPO_MAP[repo] || repo
  return {
    github: `${GITHUB_ORG}/${mappedRepo}${path}`,
    gitee: `${GITEE_ORG}/${mappedRepo}${path}`,
  }
}

/**
 * 获取 raw 文件 URL（用于 deploy.sh 等脚本下载）
 * GitHub: raw.githubusercontent.com/org/repo/branch/file
 * @param {string} repo
 * @param {string} branch
 * @param {string} filePath
 */
export async function rawFileUrl(repo, branch, filePath) {
  const mappedRepo = REPO_MAP[repo] || repo
  if (await isGithubReachable()) {
    return `${GITHUB_RAW}/${mappedRepo}/${branch}/${filePath}`
  }
  return `${GITEE_RAW}/${mappedRepo}/${branch}/${filePath}`
}

/**
 * deploy.sh 下载命令
 */
export function deployCommand() {
  return {
    github: `curl -fsSL ${GITHUB_RAW}/AgentDock/master/deploy.sh | bash`,
    gitee: `curl -fsSL ${GITEE_RAW}/AgentDock/master/deploy.sh | bash`,
  }
}

/** 强制标记 GitHub 不可达（用户手动切换时调用） */
export function forceGiteeMirror() {
  _githubReachable = false
  _lastCheck = Date.now()
}

/** 强制标记 GitHub 可达 */
export function forceGithubDirect() {
  _githubReachable = true
  _lastCheck = Date.now()
}

/** 当前是否使用 Gitee 镜像 */
export function isUsingGitee() {
  return _githubReachable === false
}

/** 手动触发一次 GitHub 可达性检测 */
export { isGithubReachable as checkGithubReachable }
