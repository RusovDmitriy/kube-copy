# 🔄 kube-copy: Kubernetes Pod File Syncer

`kube-copy` is a lightweight CLI utility to automatically synchronize local files into Kubernetes pods using `kubectl cp`. It watches your file system and Kubernetes pod lifecycle, keeping your app files up to date in development environments.

---

## 🚀 Features

- ⚡️ Real-time file sync to running pods
- 🧠 Sync on pod restarts (via label selectors)
- 📁 Sync only on changes, with debounce
- ✅ Smart pod readiness check (all containers must be `Ready`)
- 📟 Optional post-sync command (e.g. `touch` a file or trigger reload)
- 📦 Simple JSON configuration
- 🧪 Designed for rapid local dev with K8s

---

## 📦 Installation

### Manual
```bash
cargo build --release
cp target/release/kube-copy /usr/local/bin
```

---

## 📄 Configuration: `watcher.json`

```json
[
  {
    "name": "sync-common",
    "kube_context": "minikube",
    "namespace": "default",
    "label_selectors": ["app=my-app"],
    "paths": [
      { "src": "./local/path", "dest": "/app/dest" }
    ],
    "post_sync_command": "touch /app/.reload"
  }
]
```

### 🔁 `post_sync_command` (optional)
If provided, this shell command is executed **inside each pod** via `kubectl exec` after each successful sync.

Use it to:
- Trigger live-reload scripts
- Touch a watched file (e.g. for `nodemon`)
- Run arbitrary shell hooks in your container

---

## 🧰 Usage

```bash
kube-copy --config watcher.json
```

### Options

| Option            | Description                                   |
|-------------------|-----------------------------------------------|
| `--config`        | Path to config file (default: `watcher.json`) |
| `--sync-on-start` | Trigger sync to all ready pods at startup     |

---

## 🛠 How it works

- Uses `notify` to track file changes
- Uses `kube` + `kube-runtime` to track pod events
- On trigger, uses `kubectl cp` to sync file/directory
- Ensures pods are `Ready` before syncing
- Optionally executes `post_sync_command` in pod via `kubectl exec`

---

## 📌 When to Use

This tool is ideal for:

- Local development inside Kubernetes
- Replacing bind mounts / volumes in dev
- Hot reload / live update setups with fast iteration cycles

---

## 💡 Why not `ksync`, `telepresence`, or `skaffold`?

Those tools are powerful, but often:

- Heavier in dependencies
- Require sidecars or privileged Daemons
- Use custom CRDs / controllers

`kube-copy` is zero-cluster-dependency: it's just you, your files, and your pods.

---

## 📃 License

MIT © 2025 Dmitry Rusov

