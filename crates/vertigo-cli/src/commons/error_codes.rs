#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum ErrorCode {
    CantOpenWorkspace = 1,
    CantParseWorkspace = 2,
    CantFindCdylibMember = 3,
    PackageNameNotFound = 4,
    BuildFailed = 5,
    BuildPrerequisitesFailed = 6,
    WatcherError = 7,
    CantAddWatchDir = 8,
    OtherProcessAlreadyRunning = 9,
    CantReadWasmRunFromStatics = 10,
    CantReadWasmRunSourcemapFromStatics = 11,
    CantWriteToFile = 12,
    CantSpawnChildProcess = 13,
    CouldntWaitForChildProcess = 14,
    WatchPipeBroken = 16,

    NewProjectDirNotEmpty = 17,
    NewProjectCantCreateDir = 18,
    NewProjectCantUnpackStub = 19,
    NewProjectCanCreateCargoToml = 20,
    NewProjectCanWriteToCargoToml = 21,

    ServeCantFindHttpBasePath = 22,
    ServeCantReadIndexFile = 23,
    ServeCantOpenPort = 24,
    ServeWasmReadFailed = 25,
    ServeWasmCompileFailed = 26,
}
