; System installer — installs to %ProgramFiles%\Obsidian SSH for all users.
; Requires UAC elevation (RequestExecutionLevel admin); winget points
; here. The per-user variant lives in installer-user.nsi and runs
; without elevation under %LOCALAPPDATA%\Programs\Obsidian SSH.
;
; Build-time defines (override from CI):
;   /DVERSION=0.5.2          — SemVer; falls back to 0.0.0-dev
;   /DARCH=x86_64|aarch64    — used in OutFile suffix; defaults to x86_64
;   /DBINPATH=..\target\release
;                            — directory holding obsidianssh.exe; defaults
;                              to the x86_64 release path so a local
;                              `cargo build --release` followed by
;                              `makensis installer.nsi` still works
;                              without flags. `obsidianssh-mcp.exe` is no
;                              longer bundled; the app downloads it as
;                              a plugin on first enable into
;                              `%USERPROFILE%\.obsidianssh\bin\`.

!include "MUI2.nsh"

!ifndef VERSION
    !define VERSION "0.0.0-dev"
!endif
!ifndef ARCH
    !define ARCH "x86_64"
!endif
!ifndef BINPATH
    !define BINPATH "..\target\release"
!endif

Name "Obsidian SSH"
OutFile "..\obsidianssh-setup-${ARCH}.exe"
InstallDir "$PROGRAMFILES64\Obsidian SSH"
InstallDirRegKey HKLM "Software\Obsidian SSH" "InstallDir"
RequestExecutionLevel admin

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "Obsidian SSH"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "CompanyName" "ObsidianNetwork"
VIAddVersionKey "FileDescription" "Obsidian SSH client installer"
VIAddVersionKey "LegalCopyright" "AGPL-3.0-or-later"

!define MUI_ICON "..\resources\logo.ico"
!define MUI_ABORTWARNING

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

; Offer to relaunch the app from the Finish page (checked by default).
; The auto-updater exits the running app before handing off to this
; installer, so without this the app never comes back after an update.
; Relaunch through LaunchObsidianSsh (explorer.exe) instead of running
; obsidianssh.exe directly: this installer is elevated, and a direct Exec
; would start the app with the admin token.
!define MUI_FINISHPAGE_RUN
!define MUI_FINISHPAGE_RUN_TEXT "Launch Obsidian SSH"
!define MUI_FINISHPAGE_RUN_FUNCTION LaunchObsidianSsh
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

; Kill any running Obsidian SSH app + MCP instances so Windows file locks
; don't block the upgrade. AI clients (Claude Desktop / Code) keep
; obsidianssh-mcp.exe alive as a child process across Obsidian SSH app sessions,
; so the auto-updater would otherwise fail to overwrite the MCP
; binary even after the main app exits. taskkill ships with Windows;
; no extra plugin needed. Missing-process exit codes are ignored.
!macro KillRunningObsidianSsh
    nsExec::Exec '"taskkill" /F /T /IM obsidianssh-mcp.exe'
    Pop $0
    nsExec::Exec '"taskkill" /F /T /IM obsidianssh.exe'
    Pop $0
    ; Brief pause so the OS releases handles before the first File
    ; write; without it taskkill + immediate overwrite can still race
    ; into a sharing violation.
    Sleep 1000
!macroend

Function .onInit
    !insertmacro KillRunningObsidianSsh
FunctionEnd

Function un.onInit
    !insertmacro KillRunningObsidianSsh
FunctionEnd

; Relaunch the app via explorer.exe so it inherits the user's normal
; (non-elevated) token. Running "$INSTDIR\obsidianssh.exe" directly from
; this elevated installer would start the app as administrator.
Function LaunchObsidianSsh
    Exec '"$WINDIR\explorer.exe" "$INSTDIR\obsidianssh.exe"'
FunctionEnd

Section "Install"
    SetOutPath $INSTDIR

    ; Upgrading from a pre-v0.7 install: remove the bundled MCP binary
    ; if it's still there. The app now downloads MCP into
    ; `%USERPROFILE%\.obsidianssh\bin\` on demand. Missing-file is silent.
    Delete "$INSTDIR\obsidianssh-mcp.exe"

    File "${BINPATH}\obsidianssh.exe"
    File "..\resources\logo.ico"
    File "..\README.md"

    ; Create start menu shortcuts
    CreateDirectory "$SMPROGRAMS\Obsidian SSH"
    CreateShortCut "$SMPROGRAMS\Obsidian SSH\Obsidian SSH.lnk" "$INSTDIR\obsidianssh.exe" "" "$INSTDIR\logo.ico"
    CreateShortCut "$SMPROGRAMS\Obsidian SSH\Uninstall.lnk" "$INSTDIR\uninstall.exe"

    ; Desktop shortcut
    CreateShortCut "$DESKTOP\Obsidian SSH.lnk" "$INSTDIR\obsidianssh.exe" "" "$INSTDIR\logo.ico"

    ; Stamp the AppUserModelID on the start-menu + desktop shortcuts
    ; so the JumpList we build at runtime (io.obsidianssh.Obsidian SSH) is
    ; pinned to THIS .lnk, not to a Windows-derived one based on the
    ; shortcut path. Without it, JumpList recent-hosts only appears
    ; after the first launch finishes tagging the live window, and
    ; the user may never see the pinned-shortcut menu. Setting
    ; PKEY_AppUserModel_ID on a .lnk requires IPropertyStore, which
    ; needs either a custom NSIS plugin or a small bundled helper
    ; .exe (PowerShell + inline C# is brittle enough that it has
    ; broken silent installs in the past). Tracked as a follow-up
    ; rather than risk a broken installer on top of the working
    ; runtime fallback (SetCurrentProcessExplicitAppUserModelID
    ; + jumplist::tag_window).

    ; Write uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"

    ; Add INSTDIR to the system PATH so `obsidianssh` and `obsidianssh-mcp`
    ; resolve from any shell. EnVar handles dedup (no duplicate
    ; entries on reinstall) and broadcasts WM_SETTINGCHANGE so open
    ; shells pick the new value up. Errors are non-fatal.
    EnVar::SetHKLM
    EnVar::AddValue "Path" "$INSTDIR"
    Pop $0

    ; Registry — uninstall info. winget detects installed packages
    ; via these keys, so `DisplayVersion` MUST match the package
    ; version (passed in via `/DVERSION` from the release workflow)
    ; or upgrades won't be recognised.
    WriteRegStr HKLM "Software\Obsidian SSH" "InstallDir" "$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "DisplayName" "Obsidian SSH"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "DisplayIcon" "$INSTDIR\logo.ico"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "Publisher" "ObsidianNetwork"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "DisplayVersion" "${VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "URLInfoAbout" "https://github.com/ObsidianNetwork/ObsidianSSH"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "HelpLink" "https://github.com/ObsidianNetwork/ObsidianSSH"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "NoRepair" 1

    ; Registry — App Paths (makes Windows Search find the app)
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\App Paths\obsidianssh.exe" "" "$INSTDIR\obsidianssh.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\App Paths\obsidianssh.exe" "Path" "$INSTDIR"

    ; obsidianssh:// URL scheme (deep links: sync pairing, theme install).
    ; The URL arrives as argv[1]; the app forwards it to a running
    ; instance or boots with it (see deep_link.rs).
    WriteRegStr HKLM "Software\Classes\obsidianssh" "" "URL:Obsidian SSH deep link"
    WriteRegStr HKLM "Software\Classes\obsidianssh" "URL Protocol" ""
    WriteRegStr HKLM "Software\Classes\obsidianssh\DefaultIcon" "" '"$INSTDIR\obsidianssh.exe",0'
    WriteRegStr HKLM "Software\Classes\obsidianssh\shell\open\command" "" '"$INSTDIR\obsidianssh.exe" "%1"'

    ; ssh:// URL scheme. Same argv delivery, different route: a clicked
    ; link only PREFILLS the ad-hoc host editor (a web page chooses the
    ; payload), while `obsidianssh user@host` on the command line dials.
    ; Registered unconditionally: on Windows nothing else claims the
    ; scheme by default, and a user who installed something that does
    ; can reassign it in Settings > Apps > Default apps.
    WriteRegStr HKLM "Software\Classes\ssh" "" "URL:Secure Shell"
    WriteRegStr HKLM "Software\Classes\ssh" "URL Protocol" ""
    WriteRegStr HKLM "Software\Classes\ssh\DefaultIcon" "" '"$INSTDIR\obsidianssh.exe",0'
    WriteRegStr HKLM "Software\Classes\ssh\shell\open\command" "" '"$INSTDIR\obsidianssh.exe" "%1"'
SectionEnd

Section "Uninstall"
    EnVar::SetHKLM
    EnVar::DeleteValue "Path" "$INSTDIR"
    Pop $0

    Delete "$INSTDIR\obsidianssh.exe"
    ; Sweep the pre-v0.7 bundled MCP binary, if upgrading from an old
    ; install that still has it. New installs never lay this file down.
    Delete "$INSTDIR\obsidianssh-mcp.exe"
    Delete "$INSTDIR\logo.ico"
    Delete "$INSTDIR\README.md"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"

    Delete "$SMPROGRAMS\Obsidian SSH\Obsidian SSH.lnk"
    Delete "$SMPROGRAMS\Obsidian SSH\Uninstall.lnk"
    RMDir "$SMPROGRAMS\Obsidian SSH"
    Delete "$DESKTOP\Obsidian SSH.lnk"

    DeleteRegKey HKLM "Software\Obsidian SSH"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\App Paths\obsidianssh.exe"
    DeleteRegKey HKLM "Software\Classes\obsidianssh"
    DeleteRegKey HKLM "Software\Classes\ssh"
    ; Toast identity the app self-registers at runtime (util.rs::
    ; register_toast_aumid). Written per-user, so sweep HKCU even
    ; from the elevated uninstaller.
    DeleteRegKey HKCU "Software\Classes\AppUserModelId\io.obsidianssh.Obsidian SSH"
SectionEnd
