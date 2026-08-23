; CodeEraser NSIS hooks (tauri bundle > windows > nsis > installerHooks,
; v0.7.2): the installer is the GUI+CLI superset — ce.exe and
; ce-core.exe land flat in $INSTDIR (release.yml externalBin), so
; putting $INSTDIR on the machine PATH makes the CLI usable from any
; terminal. PATH surgery is delegated to PowerShell because .NET
; strings have no NSIS 8192-char truncation cliff and split/filter is
; idempotent by construction (reinstalls never stack duplicates).
;
; Since v1.0.1 the installer has a THIRD job: one install = the whole
; product. If Claude Code is on this machine (a `claude` CLI on PATH,
; or the native install at ~\.local\bin\claude.exe), POSTINSTALL
; registers the public marketplace (skymanbp/CodeEraser — the manifest
; lives at the REPO ROOT since 9f86d58; pointing at plugin/ is the
; exact stale registration that silently dropped the guard on the dev
; machine for three days) and installs/refreshes the codeeraser
; plugin. Every failure DEGRADES to a DetailPrint with the manual
; two-liner — an installer must finish installing even when offline.
; The `claude-plugin-wired` marker is written ONLY when the
; marketplace was added BY THIS INSTALLER: PREUNINSTALL keys on it, so
; uninstall removes exactly what install added and never tears down a
; registration the user made themselves (dev checkouts register the
; repo directory instead — that one is not ours to remove).
; perMachine caveat: the hook runs in the ELEVATED context, so the
; plugin lands in the elevating user's ~\.claude — on the typical
; single-admin machine that is the installing user.
;
; Two rules this file learned the hard way (review 2026-08-21, both
; reproduced first-party against a real machine):
;
;   1. NEVER interpolate $INSTDIR into PowerShell script text. An
;      apostrophe is legal in a Windows path — the example that broke
;      it was a user folder named O'Brien — and it closes the '...'
;      literal: the command fails to parse at best, and runs its tail
;      as code inside an ELEVATED installer at worst. The path travels
;      as an environment variable instead and the script never quotes
;      it: CE_INSTDIR is READ by the script, never pasted into it.
;
;   2. NEVER write the machine Path with SetEnvironmentVariable. .NET
;      writes REG_SZ, so a REG_EXPAND_SZ Path comes back with every
;      %SystemRoot%-style entry flattened to a literal and the type
;      downgraded permanently (v0.7.2 did exactly this). The value is
;      edited in place at its ORIGINAL value kind, and the broadcast
;      SetEnvironmentVariable used to do for free is sent here
;      instead: WM_WININICHANGE (0x1A) to HWND_BROADCAST (0xFFFF).
;      Literals, not header symbols, so include order cannot silence
;      the broadcast.

!macro NSIS_HOOK_POSTINSTALL
  StrCpy $0 "$INSTDIR"
  System::Call 'kernel32::SetEnvironmentVariable(t "CE_INSTDIR", t r0)i.r1'
  nsExec::ExecToLog "powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command $\"$$d=$$env:CE_INSTDIR; if (-not $$d) { exit 3 }; $$k=[Microsoft.Win32.Registry]::LocalMachine.OpenSubKey('SYSTEM\CurrentControlSet\Control\Session Manager\Environment',$$true); $$t=$$k.GetValueKind('Path'); $$p=$$k.GetValue('Path','',[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); if (($$p -split ';') -notcontains $$d) { $$k.SetValue('Path',($$p.TrimEnd(';')+';'+$$d),$$t) }; $$k.Close()$\""
  Pop $2
  StrCmp $2 "0" +2 0
    DetailPrint "CodeEraser: machine PATH not updated (exit $2) — add $INSTDIR yourself"
  SendMessage 0xFFFF 0x1A 0 "STR:Environment" /TIMEOUT=5000

  ; --- Claude Code plugin wiring (one install = both). Exit codes:
  ; 0 wired fresh (marker written) · 5 already registered, refreshed ·
  ; 10 no Claude Code · 11 marketplace add failed · 12 install failed.
  ; The marketplace-presence probe is textual on `marketplace list`
  ; (`> codeeraser` row): ANY marketplace of that name counts — a dev
  ; checkout's directory registration must be reused, not replaced.
  DetailPrint "CodeEraser: probing for Claude Code"
  nsExec::ExecToLog "powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command $\"$$c=(Get-Command claude -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1).Source; if (-not $$c) { $$p=Join-Path $$env:USERPROFILE '.local\bin\claude.exe'; if (Test-Path $$p) { $$c=$$p } }; if (-not $$c) { exit 10 }; $$l=& $$c plugin marketplace list 2>&1 | Out-String; if ($$LASTEXITCODE -ne 0) { exit 11 }; $$fresh=$$l -notmatch '(?m)^\s*>\s*codeeraser\s*$$'; if ($$fresh) { & $$c plugin marketplace add skymanbp/CodeEraser 2>&1 | Out-Null; if ($$LASTEXITCODE -ne 0) { exit 11 } }; & $$c plugin install codeeraser@codeeraser 2>&1 | Out-Null; if ($$LASTEXITCODE -ne 0) { exit 12 }; & $$c plugin update codeeraser@codeeraser 2>&1 | Out-Null; if ($$fresh) { New-Item -ItemType File -Force -Path (Join-Path $$env:CE_INSTDIR 'claude-plugin-wired') | Out-Null; exit 0 }; exit 5$\""
  Pop $2
  StrCmp $2 "0" cepi_wired
  StrCmp $2 "5" cepi_kept
  StrCmp $2 "10" cepi_nocc
  DetailPrint "CodeEraser: Claude Code plugin wiring failed (exit $2) — run yourself: claude plugin marketplace add skymanbp/CodeEraser, then claude plugin install codeeraser@codeeraser"
  Goto cepi_done
  cepi_nocc:
  DetailPrint "CodeEraser: Claude Code not detected — plugin not wired (after installing Claude Code: claude plugin marketplace add skymanbp/CodeEraser, then claude plugin install codeeraser@codeeraser)"
  Goto cepi_done
  cepi_kept:
  DetailPrint "CodeEraser: Claude Code plugin already registered — refreshed (restart Claude Code sessions to activate)"
  Goto cepi_done
  cepi_wired:
  DetailPrint "CodeEraser: Claude Code detected — plugin wired (restart Claude Code sessions to activate)"
  cepi_done:
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Un-wire ONLY what POSTINSTALL wired: no marker file, no touch.
  ; Runs BEFORE file removal so the marker is still on disk to read.
  IfFileExists "$INSTDIR\claude-plugin-wired" 0 ceppu_done
  nsExec::ExecToLog "powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command $\"$$c=(Get-Command claude -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1).Source; if (-not $$c) { $$p=Join-Path $$env:USERPROFILE '.local\bin\claude.exe'; if (Test-Path $$p) { $$c=$$p } }; if (-not $$c) { exit 10 }; & $$c plugin uninstall codeeraser@codeeraser 2>&1 | Out-Null; & $$c plugin marketplace remove codeeraser 2>&1 | Out-Null; exit 0$\""
  Pop $2
  StrCmp $2 "0" +2 0
    DetailPrint "CodeEraser: Claude Code plugin not unwired (exit $2) — remove yourself: claude plugin uninstall codeeraser@codeeraser"
  Delete "$INSTDIR\claude-plugin-wired"
  ceppu_done:
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  StrCpy $0 "$INSTDIR"
  System::Call 'kernel32::SetEnvironmentVariable(t "CE_INSTDIR", t r0)i.r1'
  nsExec::ExecToLog "powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command $\"$$d=$$env:CE_INSTDIR; if (-not $$d) { exit 3 }; $$k=[Microsoft.Win32.Registry]::LocalMachine.OpenSubKey('SYSTEM\CurrentControlSet\Control\Session Manager\Environment',$$true); $$t=$$k.GetValueKind('Path'); $$p=$$k.GetValue('Path','',[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); $$n=(($$p -split ';') | Where-Object { $$_ -ne $$d -and $$_ -ne '' }) -join ';'; $$k.SetValue('Path',$$n,$$t); $$k.Close()$\""
  Pop $2
  StrCmp $2 "0" +2 0
    DetailPrint "CodeEraser: machine PATH entry not removed (exit $2) — remove $INSTDIR yourself"
  SendMessage 0xFFFF 0x1A 0 "STR:Environment" /TIMEOUT=5000
!macroend
