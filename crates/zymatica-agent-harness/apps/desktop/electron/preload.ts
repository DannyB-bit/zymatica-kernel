import { contextBridge, ipcRenderer, webUtils } from 'electron'

contextBridge.exposeInMainWorld('zymaticaDesktop', {
  getConnection: profile => ipcRenderer.invoke('zymatica:connection', profile),
  revalidateConnection: () => ipcRenderer.invoke('zymatica:connection:revalidate'),
  touchBackend: profile => ipcRenderer.invoke('zymatica:backend:touch', profile),
  getGatewayWsUrl: profile => ipcRenderer.invoke('zymatica:gateway:ws-url', profile),
  openSessionWindow: (sessionId, opts) => ipcRenderer.invoke('zymatica:window:openSession', sessionId, opts),
  openNewSessionWindow: () => ipcRenderer.invoke('zymatica:window:openNewSession'),
  petOverlay: {
    // Main renderer → main process: window lifecycle + drag. `request` is
    // `{ bounds, screen }`; resolves with the screen bounds it actually used.
    open: request => ipcRenderer.invoke('zymatica:pet-overlay:open', request),
    close: () => ipcRenderer.invoke('zymatica:pet-overlay:close'),
    setBounds: bounds => ipcRenderer.send('zymatica:pet-overlay:set-bounds', bounds),
    setIgnoreMouse: ignore => ipcRenderer.send('zymatica:pet-overlay:ignore-mouse', ignore),
    // Flip the overlay focusable (and focus it) while the composer needs keys.
    setFocusable: focusable => ipcRenderer.send('zymatica:pet-overlay:set-focusable', focusable),
    // Main renderer → overlay (forwarded by main): push the latest pet state.
    pushState: payload => ipcRenderer.send('zymatica:pet-overlay:state', payload),
    // Overlay → main renderer (forwarded by main): pop back in / composer submit.
    control: payload => ipcRenderer.send('zymatica:pet-overlay:control', payload),
    // Overlay subscribes to state pushes.
    onState: callback => {
      const listener = (_event, payload) => callback(payload)
      ipcRenderer.on('zymatica:pet-overlay:state', listener)

      return () => ipcRenderer.removeListener('zymatica:pet-overlay:state', listener)
    },
    // Main renderer subscribes to overlay control messages.
    onControl: callback => {
      const listener = (_event, payload) => callback(payload)
      ipcRenderer.on('zymatica:pet-overlay:control', listener)

      return () => ipcRenderer.removeListener('zymatica:pet-overlay:control', listener)
    }
  },
  getBootProgress: () => ipcRenderer.invoke('zymatica:boot-progress:get'),
  getConnectionConfig: profile => ipcRenderer.invoke('zymatica:connection-config:get', profile),
  saveConnectionConfig: payload => ipcRenderer.invoke('zymatica:connection-config:save', payload),
  applyConnectionConfig: payload => ipcRenderer.invoke('zymatica:connection-config:apply', payload),
  testConnectionConfig: payload => ipcRenderer.invoke('zymatica:connection-config:test', payload),
  probeConnectionConfig: remoteUrl => ipcRenderer.invoke('zymatica:connection-config:probe', remoteUrl),
  oauthLoginConnectionConfig: remoteUrl => ipcRenderer.invoke('zymatica:connection-config:oauth-login', remoteUrl),
  oauthLogoutConnectionConfig: remoteUrl => ipcRenderer.invoke('zymatica:connection-config:oauth-logout', remoteUrl),
  // Zymatica Cloud: one portal login powers discovery + silent per-agent sign-in
  // (cloud-auto-discovery Phase 3).
  cloud: {
    status: () => ipcRenderer.invoke('zymatica:cloud:status'),
    login: () => ipcRenderer.invoke('zymatica:cloud:login'),
    logout: () => ipcRenderer.invoke('zymatica:cloud:logout'),
    discover: org => ipcRenderer.invoke('zymatica:cloud:discover', org),
    agentSignIn: dashboardUrl => ipcRenderer.invoke('zymatica:cloud:agent-sign-in', dashboardUrl)
  },
  profile: {
    get: () => ipcRenderer.invoke('zymatica:profile:get'),
    set: name => ipcRenderer.invoke('zymatica:profile:set', name)
  },
  api: request => ipcRenderer.invoke('zymatica:api', request),
  notify: payload => ipcRenderer.invoke('zymatica:notify', payload),
  requestMicrophoneAccess: () => ipcRenderer.invoke('zymatica:requestMicrophoneAccess'),
  readFileDataUrl: filePath => ipcRenderer.invoke('zymatica:readFileDataUrl', filePath),
  readFileText: filePath => ipcRenderer.invoke('zymatica:readFileText', filePath),
  selectPaths: options => ipcRenderer.invoke('zymatica:selectPaths', options),
  writeClipboard: text => ipcRenderer.invoke('zymatica:writeClipboard', text),
  saveImageFromUrl: url => ipcRenderer.invoke('zymatica:saveImageFromUrl', url),
  saveImageBuffer: (data, ext) => ipcRenderer.invoke('zymatica:saveImageBuffer', { data, ext }),
  saveClipboardImage: () => ipcRenderer.invoke('zymatica:saveClipboardImage'),
  getPathForFile: file => {
    try {
      return webUtils.getPathForFile(file) || ''
    } catch {
      return ''
    }
  },
  normalizePreviewTarget: (target, baseDir) => ipcRenderer.invoke('zymatica:normalizePreviewTarget', target, baseDir),
  watchPreviewFile: url => ipcRenderer.invoke('zymatica:watchPreviewFile', url),
  stopPreviewFileWatch: id => ipcRenderer.invoke('zymatica:stopPreviewFileWatch', id),
  setTitleBarTheme: payload => ipcRenderer.send('zymatica:titlebar-theme', payload),
  setNativeTheme: mode => ipcRenderer.send('zymatica:native-theme', mode),
  setTranslucency: payload => ipcRenderer.send('zymatica:translucency', payload),
  setPreviewShortcutActive: active => ipcRenderer.send('zymatica:previewShortcutActive', Boolean(active)),
  openExternal: url => ipcRenderer.invoke('zymatica:openExternal', url),
  openPreviewInBrowser: url => ipcRenderer.invoke('zymatica:openPreviewInBrowser', url),
  fetchLinkTitle: url => ipcRenderer.invoke('zymatica:fetchLinkTitle', url),
  sanitizeWorkspaceCwd: cwd => ipcRenderer.invoke('zymatica:workspace:sanitize', cwd),
  settings: {
    getDefaultProjectDir: () => ipcRenderer.invoke('zymatica:setting:defaultProjectDir:get'),
    setDefaultProjectDir: dir => ipcRenderer.invoke('zymatica:setting:defaultProjectDir:set', dir),
    pickDefaultProjectDir: () => ipcRenderer.invoke('zymatica:setting:defaultProjectDir:pick')
  },
  zoom: {
    // Current zoom of this window, as { level, percent }.
    get: () => ipcRenderer.invoke('zymatica:zoom:get'),
    setPercent: percent => ipcRenderer.send('zymatica:zoom:set-percent', percent),
    // Fires on every zoom change, including the Ctrl/Cmd +/-/0 shortcuts,
    // so the settings UI can stay in sync with the keyboard.
    onChanged: callback => {
      const listener = (_event, payload) => callback(payload)
      ipcRenderer.on('zymatica:zoom:changed', listener)

      return () => ipcRenderer.removeListener('zymatica:zoom:changed', listener)
    }
  },
  revealLogs: () => ipcRenderer.invoke('zymatica:logs:reveal'),
  getRecentLogs: () => ipcRenderer.invoke('zymatica:logs:recent'),
  readDir: dirPath => ipcRenderer.invoke('zymatica:fs:readDir', dirPath),
  gitRoot: startPath => ipcRenderer.invoke('zymatica:fs:gitRoot', startPath),
  revealPath: targetPath => ipcRenderer.invoke('zymatica:fs:reveal', targetPath),
  openDir: dirPath => ipcRenderer.invoke('zymatica:fs:openDir', dirPath),
  renamePath: (targetPath, newName) => ipcRenderer.invoke('zymatica:fs:rename', targetPath, newName),
  writeTextFile: (filePath, content) => ipcRenderer.invoke('zymatica:fs:writeText', filePath, content),
  trashPath: targetPath => ipcRenderer.invoke('zymatica:fs:trash', targetPath),
  git: {
    worktreeList: repoPath => ipcRenderer.invoke('zymatica:git:worktreeList', repoPath),
    worktreeAdd: (repoPath, options) => ipcRenderer.invoke('zymatica:git:worktreeAdd', repoPath, options),
    worktreeRemove: (repoPath, worktreePath, options) =>
      ipcRenderer.invoke('zymatica:git:worktreeRemove', repoPath, worktreePath, options),
    branchSwitch: (repoPath, branch) => ipcRenderer.invoke('zymatica:git:branchSwitch', repoPath, branch),
    branchList: repoPath => ipcRenderer.invoke('zymatica:git:branchList', repoPath),
    baseBranchList: repoPath => ipcRenderer.invoke('zymatica:git:baseBranchList', repoPath),
    repoStatus: repoPath => ipcRenderer.invoke('zymatica:git:repoStatus', repoPath),
    fileDiff: (repoPath, filePath) => ipcRenderer.invoke('zymatica:git:fileDiff', repoPath, filePath),
    scanRepos: (roots, options) => ipcRenderer.invoke('zymatica:git:scanRepos', roots, options),
    review: {
      list: (repoPath, scope, baseRef) => ipcRenderer.invoke('zymatica:git:review:list', repoPath, scope, baseRef),
      diff: (repoPath, filePath, scope, baseRef, staged) =>
        ipcRenderer.invoke('zymatica:git:review:diff', repoPath, filePath, scope, baseRef, staged),
      stage: (repoPath, filePath) => ipcRenderer.invoke('zymatica:git:review:stage', repoPath, filePath),
      unstage: (repoPath, filePath) => ipcRenderer.invoke('zymatica:git:review:unstage', repoPath, filePath),
      revert: (repoPath, filePath) => ipcRenderer.invoke('zymatica:git:review:revert', repoPath, filePath),
      revParse: (repoPath, ref) => ipcRenderer.invoke('zymatica:git:review:revParse', repoPath, ref),
      commit: (repoPath, message, push) => ipcRenderer.invoke('zymatica:git:review:commit', repoPath, message, push),
      commitContext: repoPath => ipcRenderer.invoke('zymatica:git:review:commitContext', repoPath),
      push: repoPath => ipcRenderer.invoke('zymatica:git:review:push', repoPath),
      shipInfo: repoPath => ipcRenderer.invoke('zymatica:git:review:shipInfo', repoPath),
      createPr: repoPath => ipcRenderer.invoke('zymatica:git:review:createPr', repoPath)
    }
  },
  terminal: {
    cwd: id => ipcRenderer.invoke('zymatica:terminal:cwd', id),
    dispose: id => ipcRenderer.invoke('zymatica:terminal:dispose', id),
    resize: (id, size) => ipcRenderer.invoke('zymatica:terminal:resize', id, size),
    start: options => ipcRenderer.invoke('zymatica:terminal:start', options),
    write: (id, data) => ipcRenderer.invoke('zymatica:terminal:write', id, data),
    onData: (id, callback) => {
      const channel = `zymatica:terminal:${id}:data`
      const listener = (_event, payload) => callback(payload)
      ipcRenderer.on(channel, listener)

      return () => ipcRenderer.removeListener(channel, listener)
    },
    onExit: (id, callback) => {
      const channel = `zymatica:terminal:${id}:exit`
      const listener = (_event, payload) => callback(payload)
      ipcRenderer.on(channel, listener)

      return () => ipcRenderer.removeListener(channel, listener)
    }
  },
  onClosePreviewRequested: callback => {
    const listener = () => callback()
    ipcRenderer.on('zymatica:close-preview-requested', listener)

    return () => ipcRenderer.removeListener('zymatica:close-preview-requested', listener)
  },
  onOpenUpdatesRequested: callback => {
    const listener = () => callback()
    ipcRenderer.on('zymatica:open-updates', listener)

    return () => ipcRenderer.removeListener('zymatica:open-updates', listener)
  },
  onDeepLink: callback => {
    const listener = (_event, payload) => callback(payload)
    ipcRenderer.on('zymatica:deep-link', listener)

    return () => ipcRenderer.removeListener('zymatica:deep-link', listener)
  },
  signalDeepLinkReady: () => ipcRenderer.invoke('zymatica:deep-link-ready'),
  onWindowStateChanged: callback => {
    const listener = (_event, payload) => callback(payload)
    ipcRenderer.on('zymatica:window-state-changed', listener)

    return () => ipcRenderer.removeListener('zymatica:window-state-changed', listener)
  },
  onFocusSession: callback => {
    const listener = (_event, sessionId) => callback(sessionId)
    ipcRenderer.on('zymatica:focus-session', listener)

    return () => ipcRenderer.removeListener('zymatica:focus-session', listener)
  },
  onNotificationAction: callback => {
    const listener = (_event, payload) => callback(payload)
    ipcRenderer.on('zymatica:notification-action', listener)

    return () => ipcRenderer.removeListener('zymatica:notification-action', listener)
  },
  onPreviewFileChanged: callback => {
    const listener = (_event, payload) => callback(payload)
    ipcRenderer.on('zymatica:preview-file-changed', listener)

    return () => ipcRenderer.removeListener('zymatica:preview-file-changed', listener)
  },
  onBackendExit: callback => {
    const listener = (_event, payload) => callback(payload)
    ipcRenderer.on('zymatica:backend-exit', listener)

    return () => ipcRenderer.removeListener('zymatica:backend-exit', listener)
  },
  // Soft gateway-mode apply finished tearing down the primary backend. Renderer
  // should wipe session lists + re-dial without a window reload.
  onConnectionApplied: callback => {
    const listener = () => callback()
    ipcRenderer.on('zymatica:connection:applied', listener)

    return () => ipcRenderer.removeListener('zymatica:connection:applied', listener)
  },
  onPowerResume: callback => {
    const listener = () => callback()
    ipcRenderer.on('zymatica:power-resume', listener)

    return () => ipcRenderer.removeListener('zymatica:power-resume', listener)
  },
  onBootProgress: callback => {
    const listener = (_event, payload) => callback(payload)
    ipcRenderer.on('zymatica:boot-progress', listener)

    return () => ipcRenderer.removeListener('zymatica:boot-progress', listener)
  },
  // First-launch bootstrap progress -- emitted by the install.ps1 stage
  // runner in main.ts (apps/desktop/electron/bootstrap-runner.ts).
  // Renderer's install overlay subscribes to live events and queries the
  // current snapshot via getBootstrapState() to recover after a devtools
  // reload mid-bootstrap.
  getBootstrapState: () => ipcRenderer.invoke('zymatica:bootstrap:get'),
  resetBootstrap: () => ipcRenderer.invoke('zymatica:bootstrap:reset'),
  repairBootstrap: () => ipcRenderer.invoke('zymatica:bootstrap:repair'),
  cancelBootstrap: () => ipcRenderer.invoke('zymatica:bootstrap:cancel'),
  onBootstrapEvent: callback => {
    const listener = (_event, payload) => callback(payload)
    ipcRenderer.on('zymatica:bootstrap:event', listener)

    return () => ipcRenderer.removeListener('zymatica:bootstrap:event', listener)
  },
  getVersion: () => ipcRenderer.invoke('zymatica:version'),
  getRemoteDisplayReason: () => ipcRenderer.invoke('zymatica:get-remote-display-reason'),
  uninstall: {
    summary: () => ipcRenderer.invoke('zymatica:uninstall:summary'),
    run: mode => ipcRenderer.invoke('zymatica:uninstall:run', { mode })
  },
  updates: {
    check: () => ipcRenderer.invoke('zymatica:updates:check'),
    apply: opts => ipcRenderer.invoke('zymatica:updates:apply', opts),
    getBranch: () => ipcRenderer.invoke('zymatica:updates:branch:get'),
    setBranch: name => ipcRenderer.invoke('zymatica:updates:branch:set', name),
    onProgress: callback => {
      const listener = (_event, payload) => callback(payload)
      ipcRenderer.on('zymatica:updates:progress', listener)

      return () => ipcRenderer.removeListener('zymatica:updates:progress', listener)
    }
  },
  themes: {
    fetchMarketplace: id => ipcRenderer.invoke('zymatica:vscode-theme:fetch', id),
    searchMarketplace: query => ipcRenderer.invoke('zymatica:vscode-theme:search', query)
  }
})
