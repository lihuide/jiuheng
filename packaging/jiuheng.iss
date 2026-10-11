#define AppName "九衡"
#define AppVersion "0.79.0"
#define AppPublisher "Jiuheng"
#define AppExeName "jiuheng.exe"

[Setup]
AppId={{4F415C27-925B-4E6A-B014-6640A34C5667}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={%LOCALAPPDATA}\Programs\Jiuheng
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
#ifdef SmokeTest
OutputDir=..\work\smoke-installer
OutputBaseFilename=Jiuheng-Smoke-x64-{#AppVersion}
CreateUninstallRegKey=no
#else
OutputDir=..\outputs
OutputBaseFilename=Jiuheng-Setup-x64-{#AppVersion}
#endif
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
UninstallDisplayName={#AppName}
SetupLogging=yes

[Languages]
Name: "chinesesimp"; MessagesFile: "ChineseSimplified.isl"

[Files]
Source: "..\work\package\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\docs\THIRD_PARTY_NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
#ifndef SmokeTest
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon
#endif

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"; GroupDescription: "快捷方式："; Flags: unchecked

[Run]
Filename: "{app}\{#AppExeName}"; Description: "启动 {#AppName}"; Flags: nowait postinstall skipifsilent
