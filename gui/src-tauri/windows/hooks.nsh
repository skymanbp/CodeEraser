; CodeEraser NSIS hooks (tauri bundle > windows > nsis > installerHooks,
; v0.7.2): the installer is the GUI+CLI superset — ce.exe and
; ce-core.exe land flat in $INSTDIR (release.yml externalBin), so
; putting $INSTDIR on the machine PATH makes the CLI usable from any
; terminal. PATH surgery is delegated to PowerShell because .NET
; strings have no NSIS 8192-char truncation cliff and split/filter is
; idempotent by construction (reinstalls never stack duplicates).
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
