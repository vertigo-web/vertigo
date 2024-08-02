VERTIGO_VERSION=`grep version crates/vertigo/Cargo.toml | head -1 | awk -F'[ ".]' '{printf("%s.%s", $4, $5)}'`
MACRO_VERSION=`grep version crates/vertigo-macro/Cargo.toml | head -1 | awk -F'[ ".]' '{printf("%s.%s", $4, $5)}'`
CLI_VERSION=`grep version crates/vertigo-cli/Cargo.toml | head -1 | awk -F'[ ".]' '{printf("%s.%s", $4, $5)}'`

TS_MAJOR=`grep "const VERTIGO_COMPAT_VERSION_MAJOR" crates/vertigo/src/driver_module/src_js/index.ts | awk -F'[ ;]' '{print($4)}'`
TS_MINOR=`grep "const VERTIGO_COMPAT_VERSION_MINOR" crates/vertigo/src/driver_module/src_js/index.ts | awk -F'[ ;]' '{print($4)}'`
JS_MAJOR=`grep -o -e "VERTIGO_COMPAT_VERSION_MAJOR = .;" crates/vertigo/src/driver_module/wasm_run.js.map | awk -F'[ ;]' '{print($3)}'`
JS_MINOR=`grep -o -e "VERTIGO_COMPAT_VERSION_MINOR = .;" crates/vertigo/src/driver_module/wasm_run.js.map | awk -F'[ ;]' '{print($3)}'`

TS_VERSION="${TS_MAJOR}.${TS_MINOR}"
JS_VERSION="${JS_MAJOR}.${JS_MINOR}"

echo VERTIGO_VERSION $VERTIGO_VERSION
echo MACRO_VERSION $MACRO_VERSION
echo CLI_VERSION $CLI_VERSION
echo TS_VERSION $TS_VERSION
echo JS_VERSION $JS_VERSION

if [ "$VERTIGO_VERSION" != "$MACRO_VERSION" ]
then
    echo "MACRO VERSION MISMATCH!" && exit 1
fi

if [ "$VERTIGO_VERSION" != "$CLI_VERSION" ]
then
    echo "CLI VERSION MISMATCH!" && exit 2
fi

if [ "$VERTIGO_VERSION" != "$TS_VERSION" ]
then
    echo "TS MAJOR/MINOR MISMATCH!" && exit 3
fi

if [ "$VERTIGO_VERSION" != "$JS_VERSION" ]
then
    echo "JS MAJOR/MINOR MISMATCH! JS not build?" && exit 4
fi

echo "OK"
