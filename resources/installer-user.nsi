; Per-user installer — installs to %LOCALAPPDATA%\Programs\Obsidian SSH for
; the current user only. Runs without UAC (RequestExecutionLevel user)
; so the auto-updater can apply updates silently. Registry entries go
; under HKCU; the system variant (installer.nsi) writes HKLM.
;
; Build-time defines mirror installer.nsi: /DVERSION, /DARCH, /DBINPATH.

!include "MUI2.nsh"
!include "LogicLib.nsh"

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
OutFile "..\obsidianssh-user-setup-${ARCH}.exe"
InstallDir "$LOCALAPPDATA\Programs\Obsidian SSH"
InstallDirRegKey HKCU "Software\Obsidian SSH" "InstallDir"
RequestExecutionLevel user

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "Obsidian SSH"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "CompanyName" "ObsidianNetwork"
VIAddVersionKey "FileDescription" "Obsidian SSH client installer (per-user)"
VIAddVersionKey "LegalCopyright" "AGPL-3.0-or-later"

!define MUI_ICON "..\resources\logo.ico"
!define MUI_ABORTWARNING

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

; Offer to relaunch the app from the Finish page (checked by default).
; The auto-updater exits the running app before handing off to this
; installer, so without this the app never comes back after an update.
; This variant runs without elevation (RequestExecutionLevel user), so
; launching obsidianssh.exe directly already gives the right token.
!define MUI_FINISHPAGE_RUN "$INSTDIR\obsidianssh.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch Obsidian SSH"
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

; Warn (don't block) if the system variant is also installed. Side-by-side
; works but creates two Start Menu entries and the user is unlikely to
; want both. Suggest manual uninstall, never silently remove the system
; copy — that would need elevation we explicitly don't have.
Function .onInit
    !insertmacro KillRunningObsidianSsh
    IfFileExists "$PROGRAMFILES64\Obsidian SSH\uninstall.exe" 0 done
        MessageBox MB_OKCANCEL|MB_ICONEXCLAMATION \
            "Obsidian SSH is already installed for all users at $PROGRAMFILES64\Obsidian SSH.$\r$\n$\r$\nInstalling the per-user version side-by-side is supported but not recommended. Uninstall the system version first via Settings > Apps, then run this installer again.$\r$\n$\r$\nClick OK to continue anyway, or Cancel to abort." \
            IDOK done
        Abort
    done:
FunctionEnd

Function un.onInit
    !insertmacro KillRunningObsidianSsh
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

    CreateDirectory "$SMPROGRAMS\Obsidian SSH"
    CreateShortCut "$SMPROGRAMS\Obsidian SSH\Obsidian SSH.lnk" "$INSTDIR\obsidianssh.exe" "" "$INSTDIR\logo.ico"
    CreateShortCut "$SMPROGRAMS\Obsidian SSH\Uninstall.lnk" "$INSTDIR\uninstall.exe"

    CreateShortCut "$DESKTOP\Obsidian SSH.lnk" "$INSTDIR\obsidianssh.exe" "" "$INSTDIR\logo.ico"

    ; AppUserModelID on .lnk: see installer.nsi for the same TODO.
    ; The runtime fallback ships the JumpList anyway, just one
    ; cold-start late.

    WriteUninstaller "$INSTDIR\uninstall.exe"

    ; Add INSTDIR to the per-user PATH (HKCU\Environment). No admin
    ; needed; takes effect for new shells, broadcast wakes existing ones.
    EnVar::SetHKCU
    EnVar::AddValue "Path" "$INSTDIR"
    Pop $0

    ; Per-user uninstall registration. The HKCU\...\Uninstall key is
    ; surfaced in Settings > Apps under "Installed for the current
    ; user only", same as VSCode's user installer. DisplayVersion
    ; matters for winget too, even on per-user installs.
    WriteRegStr HKCU "Software\Obsidian SSH" "InstallDir" "$INSTDIR"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "DisplayName" "Obsidian SSH (User)"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "DisplayIcon" "$INSTDIR\logo.ico"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "Publisher" "ObsidianNetwork"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "DisplayVersion" "${VERSION}"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "InstallLocation" "$INSTDIR"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "URLInfoAbout" "https://github.com/ObsidianNetwork/ObsidianSSH"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "HelpLink" "https://github.com/ObsidianNetwork/ObsidianSSH"
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "NoModify" 1
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH" "NoRepair" 1

    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\App Paths\obsidianssh.exe" "" "$INSTDIR\obsidianssh.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\App Paths\obsidianssh.exe" "Path" "$INSTDIR"

    ; obsidianssh:// URL scheme (deep links: sync pairing, theme install).
    ; Per-user classes shadow HKLM's, so this install wins the scheme
    ; for this user even next to a system install.
    WriteRegStr HKCU "Software\Classes\obsidianssh" "" "URL:Obsidian SSH deep link"
    WriteRegStr HKCU "Software\Classes\obsidianssh" "URL Protocol" ""
    WriteRegStr HKCU "Software\Classes\obsidianssh\DefaultIcon" "" '"$INSTDIR\obsidianssh.exe",0'
    WriteRegStr HKCU "Software\Classes\obsidianssh\shell\open\command" "" '"$INSTDIR\obsidianssh.exe" "%1"'

    ; ssh:// URL scheme. Same argv delivery, different route: a clicked
    ; link only PREFILLS the ad-hoc host editor (a web page chooses the
    ; payload), while `obsidianssh user@host` on the command line dials.
    WriteRegStr HKCU "Software\Classes\ssh" "" "URL:Secure Shell"
    WriteRegStr HKCU "Software\Classes\ssh" "URL Protocol" ""
    WriteRegStr HKCU "Software\Classes\ssh\DefaultIcon" "" '"$INSTDIR\obsidianssh.exe",0'
    WriteRegStr HKCU "Software\Classes\ssh\shell\open\command" "" '"$INSTDIR\obsidianssh.exe" "%1"'
SectionEnd

Section "Uninstall"
    EnVar::SetHKCU
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

    DeleteRegKey HKCU "Software\Obsidian SSH"
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Obsidian SSH"
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\App Paths\obsidianssh.exe"
    DeleteRegKey HKCU "Software\Classes\obsidianssh"
    DeleteRegKey HKCU "Software\Classes\ssh"
    ; Toast identity the app self-registers at runtime (util.rs::
    ; register_toast_aumid).
    DeleteRegKey HKCU "Software\Classes\AppUserModelId\io.obsidianssh.Obsidian SSH"
SectionEnd
