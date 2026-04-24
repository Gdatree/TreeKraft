; TreeKraft Installer Script for Inno Setup

#define MyAppName "TreeKraft"
#define MyAppVersion "2.5.0"
#define MyAppPublisher "TreeKraft Team"
#define MyAppExeName "treekraft.exe"

[Setup]
AppId={{TreeKraft-2.5.0-STABLE}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2
SolidCompression=yes
OutputDir=userdocs:..
OutputBaseFilename=TreeKraft_Setup
PrivilegesRequired=admin
SetupIconFile=C:\Users\Admin\trek\treekraft.ico

[Files]
Source: "C:\Users\Admin\trek\target\release\trek.exe"; DestDir: "{app}"; DestName: "treekraft.exe"; Flags: ignoreversion
Source: "C:\Users\Admin\trek\README_TREEKRAFT.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName} REPL"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName} REPL"; Filename: "{app}\{#MyAppExeName}"

[Registry]
Root: HKCR; Subkey: ".tk"; ValueType: string; ValueName: ""; ValueData: "TreeKraft.File"; Flags: uninsdeletevalue
Root: HKCR; Subkey: "TreeKraft.File"; ValueType: string; ValueName: ""; ValueData: "TreeKraft Script"; Flags: uninsdeletekey
Root: HKCR; Subkey: "TreeKraft.File\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\treekraft.exe,0"
Root: HKCR; Subkey: "TreeKraft.File\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\treekraft.exe"" ""%1"""

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Запустить TreeKraft REPL"; Flags: postinstall nowait skipifsilent

[UninstallRun]
Filename: "taskkill.exe"; Parameters: "/f /im treekraft.exe"; RunOnceId: "KillTreeKraft"