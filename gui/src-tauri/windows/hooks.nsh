; CodeEraser NSIS hooks (tauri bundle > windows > nsis > installerHooks,
; v0.7.2): the installer is the GUI+CLI superset — ce.exe and
; ce-core.exe land flat in $INSTDIR (release.yml externalBin), so
; putting $INSTDIR on the machine PATH makes the CLI usable from any
; terminal. PATH surgery is delegated to PowerShell because .NET
; strings have no NSIS 8192-char truncation cliff and split/filter is
; idempotent by construction (reinstalls never stack duplicates).
;
; Since v1.0.1 the installer has a THIRD job: one install = the whole
; product. If Claude Code is on this machine, the plugin gets wired.
; Since v1.7.0 that wiring is `ce setup` (cli/src/setup/) — one Rust
; body the AppImage and dmg users run by hand and this hook runs for
; the Windows user — so the marketplace source, the plugin target and
; the `claude-plugin-wired` marker are spelled in exactly one place
; and cli/tests/gui/installer_wiring.js reads them off that source.
; The marker is written ONLY when the marketplace was added BY THIS
; INSTALL: PREUNINSTALL keys on it, so uninstall removes exactly what
; install added and never tears down a registration the user made
; themselves. Every failure DEGRADES to a DetailPrint — an installer
; must finish installing even when offline — and `ce setup` prints its
; own bilingual verdict line into this log before the legend below.
; perMachine caveat: the hook runs in the ELEVATED context, so the
; plugin would land in the elevating user's ~\.claude; `ce setup`
; compares that account with the console user and refuses by name
; (exit 13) when they differ instead of wiring the wrong home.
;
; Three rules this file learned the hard way (the first two from the
; review of 2026-08-21, the third measured on 2026-11-16, all three
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
;
;   3. NEVER let the template copy ce.exe / ce-core.exe over a file a
;      process holds. The bundler asks "is the app running" only for
;      the GUI program; the external binaries are a plain `File /a`,
;      and a silent install over a running copy (the daemon of any
;      open Claude Code session may hold either) exited 0 and left the
;      old bytes in place (2026-11-16, per-user probe of that template
;      line). PREINSTALL renames each of them aside first, which is
;      what the updater relies on too (cli/src/update/apply.rs). The
;      twin name carries one GetTickCount reading per install: the
;      twin of an earlier upgrade may still be mapped by a session
;      that is still open, and a fixed name could not be reused. The
;      twin is copied back to the real name at once, so the name never
;      goes missing if the installer stops before its own copy.
;      Twins are swept at the end of every install and at uninstall;
;      one a process still maps stays until the next sweep.

; The retired twins of earlier runs: this installer's <name>.<tick>.old
; and the updater's <name>.old. Best effort; a mapped twin stays.
!macro CE_SWEEP_RETIRED
  Delete "$INSTDIR\ce.*.old"
  Delete "$INSTDIR\ce-core.*.old"
  Delete "$INSTDIR\ce.old"
  Delete "$INSTDIR\ce-core.old"
  ClearErrors
!macroend

; Rename $INSTDIR\<NAME>.exe aside (rule 3) and copy it back; a rename
; that fails changes nothing, a copy that fails is undone. $R9 holds
; the tick PREINSTALL read.
!macro CE_RETIRE_SIDECAR NAME ID
  IfFileExists "$INSTDIR\${NAME}.exe" 0 ce_retire_${ID}_done
  ClearErrors
  Rename "$INSTDIR\${NAME}.exe" "$INSTDIR\${NAME}.$R9.old"
  IfErrors ce_retire_${ID}_done
  CopyFiles /SILENT "$INSTDIR\${NAME}.$R9.old" "$INSTDIR\${NAME}.exe"
  IfErrors 0 ce_retire_${ID}_done
  Rename "$INSTDIR\${NAME}.$R9.old" "$INSTDIR\${NAME}.exe"
  ce_retire_${ID}_done:
  ClearErrors
!macroend

; Called by the template after SetOutPath $INSTDIR, before it copies
; files: every external binary is retired, on every install (one path,
; exercised by every upgrade, not only when a probe says it is held).
!macro NSIS_HOOK_PREINSTALL
  Push $R9
  ; the DWORD reads as a negative int after 24.8 days of uptime: %u
  System::Call 'kernel32::GetTickCount()i.R9'
  IntFmt $R9 "%u" $R9
  !insertmacro CE_SWEEP_RETIRED
  !insertmacro CE_RETIRE_SIDECAR ce ce
  !insertmacro CE_RETIRE_SIDECAR ce-core core
  Pop $R9
!macroend

!macro NSIS_HOOK_POSTINSTALL
  !insertmacro CE_SWEEP_RETIRED
  StrCpy $0 "$INSTDIR"
  System::Call 'kernel32::SetEnvironmentVariable(t "CE_INSTDIR", t r0)i.r1'
  nsExec::ExecToLog "powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command $\"$$d=$$env:CE_INSTDIR; if (-not $$d) { exit 3 }; $$k=[Microsoft.Win32.Registry]::LocalMachine.OpenSubKey('SYSTEM\CurrentControlSet\Control\Session Manager\Environment',$$true); $$t=$$k.GetValueKind('Path'); $$p=$$k.GetValue('Path','',[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); if (($$p -split ';') -notcontains $$d) { $$k.SetValue('Path',($$p.TrimEnd(';')+';'+$$d),$$t) }; $$k.Close()$\""
  Pop $2
  StrCmp $2 "0" +2 0
    DetailPrint "CodeEraser: machine PATH not updated (exit $2) — add $INSTDIR yourself"
  SendMessage 0xFFFF 0x1A 0 "STR:Environment" /TIMEOUT=5000

  ; --- Claude Code plugin wiring (one install = both), delegated to
  ; `ce setup`. Its exit codes: 0 wired fresh (marker written) · 5
  ; already registered, refreshed · 10 no Claude Code · 11 marketplace
  ; add failed · 12 install failed · 13 elevated account is not the
  ; logged-in user (nothing wired). 0 and 5 are success; the rest are
  ; degraded and `ce setup` has already printed what to run by hand —
  ; this hook spells no claude command of its own (the gate holds it
  ; to that), so the recovery line can never go stale here.
  DetailPrint "CodeEraser: wiring the Claude Code plugin (ce setup)"
  nsExec::ExecToLog '"$INSTDIR\ce.exe" setup'
  Pop $2
  StrCmp $2 "0" cepi_done
  StrCmp $2 "5" cepi_done
  DetailPrint "CodeEraser: Claude Code plugin not wired (ce setup exit $2) — see the line above; run `ce setup` from your own account once Claude Code is installed"
  cepi_done:
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Un-wire ONLY what POSTINSTALL wired: no marker file, no touch.
  ; Runs BEFORE file removal so the marker and ce.exe are still on
  ; disk; `ce setup --unwire` keys on the same marker and deletes it,
  ; the Delete below covers the exit-13 refusal (the directory is
  ; about to go either way). The sweep runs first, marker or not.
  !insertmacro CE_SWEEP_RETIRED
  IfFileExists "$INSTDIR\claude-plugin-wired" 0 ceppu_done
  nsExec::ExecToLog '"$INSTDIR\ce.exe" setup --unwire'
  Pop $2
  StrCmp $2 "0" +2 0
    DetailPrint "CodeEraser: Claude Code plugin not unwired (ce setup --unwire exit $2) — remove yourself: claude plugin uninstall codeeraser@codeeraser, then claude plugin marketplace remove codeeraser"
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
