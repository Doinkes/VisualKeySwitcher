# Package manifests

Ready-to-use manifests for **winget** and **Scoop**. Both point to the
`v1.0.0` release asset and use its SHA-256.

## winget

Package ID: `Doinkes.VisualKeySwitcher`

The three YAML files are ready:

- `winget/Doinkes.VisualKeySwitcher.yaml`
- `winget/Doinkes.VisualKeySwitcher.installer.yaml`
- `winget/Doinkes.VisualKeySwitcher.locale.en-US.yaml`

How to publish:

1. Fork [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs).
2. Copy the three files into
   `manifests/d/Doinkes/VisualKeySwitcher/1.0.0/` in the fork.
3. Commit, push and open a pull request. The validation bot checks the
   manifest and the installer URL automatically.
4. After the merge, users install it with:

```powershell
winget install Doinkes.VisualKeySwitcher
```

The installer is a portable zip: winget extracts `VisualKeySwitcher\` and
creates a `visualkeyswitcher` command alias. The app still stores its settings
in `%APPDATA%\VisualKeySwitcher`, so updates keep the settings.

## Scoop

Manifest: `scoop/VisualKeySwitcher.json`

How to publish (simplest option — your own bucket):

1. Create a repository named `scoop-bucket` under your account.
2. Put the manifest at `bucket/VisualKeySwitcher.json` there.
3. Users install it with:

```powershell
scoop bucket add doinkes https://github.com/Doinkes/scoop-bucket
scoop install visualkeyswitcher
```

You can also submit the manifest to `ScoopInstaller/Extras` as a pull request;
check their contribution rules first.

## Updating the 1.0 release

The project keeps a single release version, `1.0`. If the exe changes:

1. Rebuild and create `VisualKeySwitcher-1.0.0.zip` (top-level
   `VisualKeySwitcher` folder with the exe, `README.txt` and `sound_pack`).
2. Replace the asset in the existing `v1.0.0` release and update its SHA-256.
3. Update `InstallerSha256` in the winget installer manifest and `hash` in the
   Scoop manifest. The version and URL stay the same.
