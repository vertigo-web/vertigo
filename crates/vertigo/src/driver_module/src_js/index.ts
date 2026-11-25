import { boot } from "./boot";

// vertigo-cli compatibility version, change together with package version.
const VERTIGO_COMPAT_VERSION_MAJOR = 0;
const VERTIGO_COMPAT_VERSION_MINOR = 14;

// The runtime is the chunk rollup makes of what is imported here, see `boot.ts`.
boot(() => import("./runtime"), VERTIGO_COMPAT_VERSION_MAJOR, VERTIGO_COMPAT_VERSION_MINOR);
