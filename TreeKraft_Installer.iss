; TreeKraft Installer Script for Inno Setup
; ============================================
; Релиз: u1 (пакет 1.0.0)
; ============================================

#define MyAppName "TreeKraft"
#define MyAppVersion "1.0.0"
#define MyAppRelease "u1"
#define MyAppPublisher "Gdatree"
#define MyAppExeName "treekraft.exe"

[Setup]
AppId={{TreeKraft-u1-STABLE}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL=https://treekraft.tilda.ws/
AppSupportURL=https://treekraft.tilda.ws/
AppUpdatesURL=https://treekraft.tilda.ws/
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2
SolidCompression=yes
OutputDir=C:\Users\Admin
OutputBaseFilename=TreeKraft instaler
PrivilegesRequired=admin
UninstallFilesDir={app}\uninstall
RestartApplications=false
CloseApplications=false
ArchitecturesInstallIn64BitMode=x64compatible
ChangesEnvironment=yes

[Files]
; Основной исполняемый файл (собрать: cargo build --release)
Source: "C:\Users\Admin\trek\target\release\treekraft.exe"; DestDir: "{app}"; Flags: ignoreversion
; Документация
Source: "C:\Users\Admin\trek\docs\treekraft.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "C:\Users\Admin\trek\docs\COMMANDS.md"; DestDir: "{app}"; Flags: ignoreversion
; Иконка для .tk файлов
Source: "C:\Users\Admin\trek\assets\tk-file.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
; Ярлык в меню Пуск
Name: "{group}\{#MyAppName} REPL"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
; Ярлык на рабочем столе
Name: "{autodesktop}\{#MyAppName} REPL"; Filename: "{app}\{#MyAppExeName}"

[Registry]
; Ассоциация .tk файлов
Root: HKCR; Subkey: ".tk"; ValueType: string; ValueName: ""; ValueData: "TreeKraft.File"; Flags: uninsdeletevalue
Root: HKCR; Subkey: "TreeKraft.File"; ValueType: string; ValueName: ""; ValueData: "TreeKraft Script"; Flags: uninsdeletekey
Root: HKCR; Subkey: "TreeKraft.File\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\tk-file.ico,0"
Root: HKCR; Subkey: "TreeKraft.File\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\treekraft.exe"" ""%1"""

; Ассоциация .trek файлов (запасное расширение)
Root: HKCR; Subkey: ".trek"; ValueType: string; ValueName: ""; ValueData: "TreeKraft.File"; Flags: uninsdeletevalue

; Добавление в PATH (чтобы можно было запускать treekraft из любого места)
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddPath('{app}')

[Run]
; Запуск после установки
Filename: "{app}\{#MyAppExeName}"; Description: "Запустить TreeKraft REPL"; Flags: postinstall nowait skipifsilent

[UninstallRun]
; Закрыть процесс перед удалением
Filename: "taskkill.exe"; Parameters: "/f /im treekraft.exe"; RunOnceId: "KillTreeKraft"

[Code]
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', OrigPath)
  then begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
end;
