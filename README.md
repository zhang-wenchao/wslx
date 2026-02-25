# wslx

Minimal Windows → WSL command forwarder that reduces the friction of maintaining two separate toolchains on Windows and WSL.  
It maps Windows-style paths in arguments to WSL paths and then executes the command inside WSL.

## Usage
1. Build the Windows binary.
2. Rename `wslx.exe` to the command you want to forward (for example `git.exe`).
3. Run it normally from Windows.

Example:
```powershell
git status
git -C "C:\work\repo" status
```

Notes:
- Only arguments are converted. Output is passed through unchanged.
