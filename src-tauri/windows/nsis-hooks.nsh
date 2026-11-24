; Survey Copilot NSIS UX hooks.
; These hooks only create/remove the Start Menu uninstall shortcut.
; They intentionally do not delete APPDATA, LOCALAPPDATA or Windows Credential Manager secrets.

!macro NSIS_HOOK_POSTINSTALL
  CreateDirectory "$SMPROGRAMS\Survey Copilot"
  CreateShortcut "$SMPROGRAMS\Survey Copilot\Uninstall Survey Copilot.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\${MAINBINARYNAME}.exe" 0
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$SMPROGRAMS\Survey Copilot\Uninstall Survey Copilot.lnk"
!macroend
