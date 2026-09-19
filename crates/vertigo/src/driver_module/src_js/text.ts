// The one `TextDecoder` and the one `TextEncoder` for the whole driver.
//
// Both are stateless for our use - `decode` on a complete buffer, `encode` to a fresh array -
// so a module-level instance is shared safely. Constructing them per call was measurable in
// two places: `jsjson` built an encoder for every string *and every object key*, twice over,
// since sizing and writing are separate passes; the panic handler built a decoder per panic.

export const decoder = new TextDecoder("utf-8");
export const encoder = new TextEncoder();
