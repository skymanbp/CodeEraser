; CodeEraser NSIS hooks (tauri bundle > windows > nsis > installerHooks,
; v0.7.2): the installer is the GUI+CLI superset — ce.exe and
; ce-core.exe land flat in $INSTDIR (release.yml externalBin), so
; putting $INSTDIR on the machine PATH makes the CLI usable from any
; terminal, and the Claude Code plugin's third resolver leg (PATH)
; reuses these binaries instead of downloading a second copy.
;
; PATH surgery is delegated to PowerShell on purpose: .NET strings
; have no NSIS 8192-char truncation cliff, split/filter is idempotent
; by construction (reinstalls never stack duplicates), and
; SetEnvironmentVariable(..., 'Machine') both writes HKLM and
; broadcasts WM_SETTINGCHANGE. Both hooks run elevated (perMachine).

!macro NSIS_HOOK_POSTINSTALL
  nsExec::ExecToLog "powershell -NoProfile -ExecutionPolicy Bypass -Command $\"$$p = [Environment]::GetEnvironmentVariable('Path','Machine'); if (($$p -split ';') -notcontains '$INSTDIR') { [Environment]::SetEnvironmentVariable('Path', $$p.TrimEnd(';') + ';' + '$INSTDIR', 'Machine') }$\""
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  nsExec::ExecToLog "powershell -NoProfile -ExecutionPolicy Bypass -Command $\"$$p = [Environment]::GetEnvironmentVariable('Path','Machine'); $$n = ($$p -split ';' | Where-Object { $$_ -ne '$INSTDIR' -and $$_ -ne '' }) -join ';'; [Environment]::SetEnvironmentVariable('Path', $$n, 'Machine')$\""
!macroend
